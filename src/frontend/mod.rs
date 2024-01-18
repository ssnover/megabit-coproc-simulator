use yew::prelude::*;

mod debug_led;
use debug_led::DebugLed;
mod matrix;
use matrix::Canvas;
mod user_button;
use user_button::UserButton;
mod websocket_provider;
use websocket_provider::WebsocketProvider;

#[function_component(App)]
pub fn app() -> Html {
    let bin_state = use_state(|| false);
    let onclick = {
        let bin_state = bin_state.clone();
        Callback::from(move |_| bin_state.set(!*bin_state))
    };
    let renderer_cb = {
        let bin_state = bin_state.clone();
        Callback::from(move |canvas| {
            matrix::draw(canvas, *bin_state, *bin_state);
        })
    };

    html! {
        <WebsocketProvider>
            <h1>{ "Megabit Coproc Simulator" }</h1>
            <DebugLed/> <UserButton/>
            <button {onclick}>{"Swap"}</button>
            <Canvas
                style={""}
                renderer={renderer_cb}
            />
        </WebsocketProvider>
    }
}
