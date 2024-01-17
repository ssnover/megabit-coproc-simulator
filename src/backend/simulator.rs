use std::time::Duration;

use async_channel::{Receiver, Sender};

pub async fn run(
    from_ws: Receiver<String>,
    from_serial: Receiver<Vec<u8>>,
    to_ws: Sender<String>,
    to_serial: Sender<Vec<u8>>,
) {
    while let Ok(msg) = from_serial.recv().await {
        if msg.len() >= 2 {
            if msg[0] == 0xde && msg[1] == 0x00 && msg.len() >= 3 {
                if msg[2] == 0x00 {
                    tracing::info!("Got request to disable debug LED");
                } else {
                    tracing::info!("Got request to enable debug LED");
                }
                to_serial.send(vec![0xde, 0x01, 0x00]).await.unwrap();
            }
        }
    }
}
