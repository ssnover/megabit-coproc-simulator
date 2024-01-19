use futures::{SinkExt, StreamExt};
use gloo::{
    net::websocket::{futures::WebSocket, Message},
    utils::window,
};
use std::{cell::RefCell, rc::Rc, time::Duration};
use wasm_bindgen_futures::spawn_local;
use yew::{platform::time::sleep, prelude::*};

use crate::messages::{FrontendStarted, SetDebugLed, SetRgbLed, SimMessage};

#[derive(Clone, PartialEq)]
pub struct WebsocketHandle {
    send_message: Callback<String>,
}

impl WebsocketHandle {
    pub fn send_message(&self, msg: String) {
        self.send_message.emit(msg);
    }
}

#[function_component]
pub fn WebsocketProvider(props: &WebsocketProviderProps) -> Html {
    let connection = use_state(|| {
        let hostname = if let (Ok(hostname), Ok(port)) =
            (window().location().hostname(), window().location().port())
        {
            format!("{hostname}:{port}")
        } else {
            log::error!("Failed to retrieve the hostname");
            String::new()
        };

        let ws = WebSocket::open(&format!("ws://{hostname}/ws")).unwrap();
        let (writer, reader) = ws.split();

        (Rc::new(RefCell::new(writer)), Rc::new(RefCell::new(reader)))
    });
    use_effect_with((), {
        let led_state_setter = props.set_led_state.clone();
        let rgb_state_setter = props.set_rgb_state.clone();
        let connection = connection.clone();
        move |()| {
            spawn_local(async move {
                if let Err(err) = connection
                    .0
                    .try_borrow_mut()
                    .unwrap()
                    .send(Message::Text(
                        serde_json::to_string(&SimMessage::FrontendStarted(FrontendStarted {}))
                            .unwrap(),
                    ))
                    .await
                {
                    log::error!("Failed to send startup ws message: {err}");
                }

                loop {
                    let mut reader = connection.1.try_borrow_mut().unwrap();
                    if let Some(Ok(msg)) = reader.next().await {
                        match msg {
                            Message::Text(msg) => {
                                log::info!("Got message: {msg}");
                                handle_simulator_message(msg, &led_state_setter, &rgb_state_setter);
                            }
                            _ => log::info!("Got bytes"),
                        }
                    }
                    sleep(Duration::from_millis(30)).await;
                }
            });
        }
    });

    let send_message = {
        let connection = connection.clone();
        move |msg: String| {
            let connection = connection.clone();
            spawn_local(async move {
                if let Err(err) = connection
                    .0
                    .try_borrow_mut()
                    .unwrap()
                    .send(Message::Text(msg))
                    .await
                {
                    log::error!("Failed to send ws message: {err}");
                }
            });
        }
    }
    .into();
    let context = WebsocketHandle { send_message };

    html! {
        <ContextProvider<WebsocketHandle> {context}>{props.children.clone()}</ContextProvider<WebsocketHandle>>
    }
}

#[derive(Properties, PartialEq)]
pub struct WebsocketProviderProps {
    pub set_led_state: UseStateSetter<bool>,
    pub set_rgb_state: UseStateSetter<(u8, u8, u8)>,
    pub children: Children,
}

#[hook]
pub fn use_websocket() -> WebsocketHandle {
    use_context().unwrap()
}

fn handle_simulator_message(
    msg: String,
    led_state_setter: &UseStateSetter<bool>,
    rgb_state_setter: &UseStateSetter<(u8, u8, u8)>,
) {
    if let Ok(msg) = serde_json::from_str::<SimMessage>(&msg) {
        match msg {
            SimMessage::SetDebugLed(SetDebugLed { new_state }) => led_state_setter.set(new_state),
            SimMessage::SetRgbLed(SetRgbLed { r, g, b }) => rgb_state_setter.set((r, g, b)),
            _ => log::warn!("Unhandled sim message: {msg:?}"),
        }
    } else {
        log::error!("Failed to parse sim message: {msg}")
    }
}
