use futures::{SinkExt, StreamExt};
use gloo::{
    net::websocket::{futures::WebSocket, Message},
    utils::window,
};
use std::{cell::RefCell, rc::Rc, time::Duration};
use wasm_bindgen_futures::spawn_local;
use yew::{html::ChildrenProps, platform::time::sleep, prelude::*};

use crate::messages::{FrontendStarted, SimMessage};

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
pub fn WebsocketProvider(ChildrenProps { children }: &ChildrenProps) -> Html {
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
                            Message::Text(msg) => log::info!("Got message: {msg}"),
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
        <ContextProvider<WebsocketHandle> {context}>{children.clone()}</ContextProvider<WebsocketHandle>>
    }
}

#[hook]
pub fn use_websocket() -> WebsocketHandle {
    use_context().unwrap()
}
