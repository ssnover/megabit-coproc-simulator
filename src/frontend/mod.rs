use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

mod matrix;
use matrix::Canvas;

#[function_component(App)]
pub fn app() -> Html {
    let bin_state = use_state(|| false);
    let onclick = {
        let bin_state = bin_state.clone();
        Callback::from(move |_| bin_state.set(!*bin_state))
    };
    let renderer_cb = {
        let bin_state = bin_state.clone();
        Callback::from(move |canvas: HtmlCanvasElement| {
            let interface: CanvasRenderingContext2d = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into()
                .unwrap();
            interface.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

            let (color, start) = if *bin_state {
                ("#ff0000", 0)
            } else {
                ("#0000ff", 1)
            };
            let color = JsValue::from(color);
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
        })
    };

    html! {
        <>
            <h1>{ "Hello World" }</h1>
            <button {onclick}>{"Swap"}</button>
            <Canvas
                style={""}
                renderer={renderer_cb}
            />
        </>
    }
}
