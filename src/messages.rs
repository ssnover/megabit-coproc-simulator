use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum SimMessage {
    FrontendStarted(FrontendStarted),
}

#[derive(Serialize, Deserialize)]
pub struct FrontendStarted {}
