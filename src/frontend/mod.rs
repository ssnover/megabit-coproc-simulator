use yew::prelude::*;

mod debug_led;
use debug_led::DebugLed;
mod matrix;
use matrix::Canvas;
mod rgb_led;
use rgb_led::RgbLed;
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
    let led_state = use_state(|| false);
    let led_state_setter = led_state.setter();

    let rgb_state = use_state(|| (0, 0, 0));
    let rgb_state_setter = rgb_state.setter();

    html! {
        <WebsocketProvider set_led_state={led_state_setter} set_rgb_state={rgb_state_setter}>
            <h1>{ "Megabit Coproc Simulator" }</h1>
            <UserButton/>
            <DebugLed {led_state}/>
            <RgbLed {rgb_state}/>
            <button {onclick}>{"Swap"}</button>
            <Canvas
                style={""}
                renderer={renderer_cb}
            />
        </WebsocketProvider>
    }
}
