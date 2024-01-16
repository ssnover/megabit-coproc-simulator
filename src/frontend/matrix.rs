use gloo::{events::EventListener, utils::window};
use std::ops::Deref;
use web_sys::HtmlCanvasElement;
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
