use gloo::{events::EventListener, utils::window};
use std::ops::Deref;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

#[function_component(Canvas)]
pub fn canvas(props: &CanvasProperties) -> Html {
    let node_ref = NodeRef::default();
    let is_first_render = use_state(|| true);
    let style = props.style.clone().unwrap_or(String::new());
    let display_size = use_state(|| (64 * 8, 32 * 8));

    let size_listen_event_state = use_state(|| EventListener::new(&window(), "resize", |_| ()));

    let node_ref_clone = node_ref.clone();
    let display_size_handle = display_size.clone();
    let renderer = props.renderer.clone();

    use_effect(move || {
        if let Some(canvas) = node_ref_clone.cast::<HtmlCanvasElement>() {
            if *is_first_render {
                is_first_render.set(false);
                let canvas = canvas.clone();

                display_size_handle.set((canvas.client_width(), canvas.client_height()));

                size_listen_event_state.set(EventListener::new(&window(), "resize", move |_| {
                    display_size_handle.set((canvas.client_width(), canvas.client_height()));
                }))
            }

            renderer.emit(canvas);
        }

        || ()
    });

    html! {
        <canvas
            style={style}
            width={display_size.clone().deref().0.to_string()}
            height={display_size.deref().1.to_string()}
            ref={node_ref}
        >
        </canvas>
    }
}

#[derive(Properties, PartialEq)]
pub struct CanvasProperties {
    pub renderer: Callback<HtmlCanvasElement>,
    pub style: Option<String>,
}

pub fn draw(canvas: HtmlCanvasElement, red: bool, evens: bool) {
    let interface: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();
    interface.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

    let color = JsValue::from(if red { "#ff0000" } else { "#0000ff" });
    let start = if evens { 0 } else { 1 };
    const SIZE_OF_ELEMENT: u32 = 8;

    interface.set_fill_style(&color);
    for row in (0..(canvas.height() / SIZE_OF_ELEMENT)).into_iter() {
        let start = ((if row % 2 == 0 { 0 } else { 1 }) + start) % 2;
        for col in (start..(canvas.width() / SIZE_OF_ELEMENT))
            .into_iter()
            .step_by(2)
        {
            interface.fill_rect(
                (col * SIZE_OF_ELEMENT) as f64,
                (row * SIZE_OF_ELEMENT) as f64,
                SIZE_OF_ELEMENT as f64,
                SIZE_OF_ELEMENT as f64,
            );
        }
    }
}
