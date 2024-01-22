use crate::messages::{SetDebugLed, SetRgbLed, SimMessage};
use async_channel::{Receiver, Sender};

pub async fn run(
    from_ws: Receiver<String>,
    from_serial: Receiver<Vec<u8>>,
    to_ws: Sender<String>,
    to_serial: Sender<Vec<u8>>,
) {
    tokio::select! {
        _ = handle_serial_message(from_serial, to_ws, to_serial.clone()) => {
            tracing::info!("Serial handler exited");
        },
        _ = handle_ws_message(from_ws, to_serial) => {
            tracing::info!("Websocket message handler exited");
        }
    }
}

async fn handle_serial_message(
    from_serial: Receiver<Vec<u8>>,
    to_ws: Sender<String>,
    to_serial: Sender<Vec<u8>>,
) {
    while let Ok(msg) = from_serial.recv().await {
        if msg.len() >= 2 {
            if msg[0] == 0xde && msg[1] == 0x00 && msg.len() >= 3 {
                let new_state = msg[2] != 0x00;
                if let Ok(msg) =
                    serde_json::to_string(&SimMessage::SetDebugLed(SetDebugLed { new_state }))
                {
                    let _ = to_ws.send(msg).await;
                }
                if !new_state {
                    tracing::info!("Got request to disable debug LED");
                } else {
                    tracing::info!("Got request to enable debug LED");
                }
                to_serial.send(vec![0xde, 0x01, 0x00]).await.unwrap();
            }
            if msg[0] == 0xde && msg[1] == 0x02 && msg.len() >= 5 {
                let (r, g, b) = (msg[2], msg[3], msg[4]);
                if let Ok(msg) =
                    serde_json::to_string(&SimMessage::SetRgbLed(SetRgbLed { r, g, b }))
                {
                    let _ = to_ws.send(msg).await;
                }
                to_serial.send(vec![0xde, 0x03, 0x00]).await.unwrap();
            }
        }
    }
}

async fn handle_ws_message(from_ws: Receiver<String>, to_serial: Sender<Vec<u8>>) {
    while let Ok(msg_str) = from_ws.recv().await {
        if let Ok(msg) = serde_json::from_str::<SimMessage>(&msg_str) {
            match msg {
                SimMessage::ReportButtonPress => {
                    tracing::debug!("Sending button press notification");
                    to_serial.send(vec![0xde, 0x04]).await.unwrap();
                }
                _ => {
                    tracing::warn!("Got unexpected message from the frontend: {msg_str}");
                }
            }
        }
    }
}
