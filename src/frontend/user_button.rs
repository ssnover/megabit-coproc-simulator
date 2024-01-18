use crate::frontend::websocket_provider::use_websocket;
use yew::prelude::*;

#[function_component(UserButton)]
pub fn user_button(_props: &UserButtonProperties) -> Html {
    let ws = use_websocket();
    let node_ref = NodeRef::default();

    let on_press = {
        let ws = ws.clone();
        Callback::from(move |_| ws.send_message("Button pressed".to_string()))
    };

    html! {
        <button
            width="20px"
            height="20px"
            ref={node_ref}
            onclick={on_press}
        >
            <p>{"User Button"}</p>
        </button>
    }
}

#[derive(Properties, PartialEq)]
pub struct UserButtonProperties {}
