use yew::prelude::*;

#[function_component(DebugLed)]
pub fn debug_led(props: &DebugLedProperties) -> Html {
    let node_ref = NodeRef::default();

    html! {
        <div
            width="20px"
            height="20px"
            ref={node_ref}
        >
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct DebugLedProperties {}
