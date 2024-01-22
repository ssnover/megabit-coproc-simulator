use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "msg", content = "data")]
pub enum SimMessage {
    FrontendStarted(FrontendStarted),
    SetDebugLed(SetDebugLed),
    SetRgbLed(SetRgbLed),
    ReportButtonPress,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FrontendStarted {}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetDebugLed {
    pub new_state: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetRgbLed {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
