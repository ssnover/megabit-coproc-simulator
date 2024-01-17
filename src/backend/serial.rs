use async_channel::{Receiver, Sender};
use nix::pty::{self, OpenptyResult};
use std::{
    io,
    os::fd::{AsRawFd, FromRawFd, IntoRawFd},
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
};

pub struct VirtualSerial {
    host_file: File,
    _host_path: PathBuf,
    _device_file: File,
    device_path: PathBuf,
}

impl VirtualSerial {
    pub fn new(symlink_path: impl AsRef<Path>) -> io::Result<Self> {
        let OpenptyResult { master, slave } = pty::openpty(None, None).expect("Failed to open pty");
        let serial = Self {
            _host_path: nix::unistd::ttyname(master.as_raw_fd()).expect("Valid fd for pty"),
            host_file: unsafe { File::from_raw_fd(master.into_raw_fd()) },
            device_path: nix::unistd::ttyname(slave.as_raw_fd()).expect("Valid fd for pty"),
            _device_file: unsafe { File::from_raw_fd(slave.into_raw_fd()) },
        };
        tracing::debug!("Host path: {}", serial._host_path.display());
        if symlink_path.as_ref().exists() {
            std::fs::remove_file(symlink_path.as_ref())?;
        }
        tracing::info!(
            "Creating symlink to {} at {}",
            serial.device_path.display(),
            symlink_path.as_ref().display()
        );
        std::os::unix::fs::symlink(&serial.device_path, symlink_path)?;
        Ok(serial)
    }

    pub async fn listen(self, to_simulator: Sender<Vec<u8>>, from_simulator: Receiver<Vec<u8>>) {
        let (mut reader, mut writer) = tokio::io::split(self.host_file);

        // tokio::join!(
        //     Self::handle_incoming_serial_bytes(reader, to_simulator),
        //     Self::handle_simulator_packet(writer, from_simulator)
        // );
        tokio::join!(
            async {
                let mut incoming_buffer = Vec::with_capacity(1024);
                while let Ok(_bytes_read) = reader.read_buf(&mut incoming_buffer).await {
                    if let Ok(decoded_data) = cobs::decode_vec(&incoming_buffer[..]) {
                        tracing::debug!(
                    "Decoded a payload of {} bytes from buffer of {} bytes. Whole buffer: {incoming_buffer:02x?}",
                    decoded_data.len(),
                    incoming_buffer.len()
                );
                        if let Err(err) = to_simulator.send(decoded_data).await {
                            tracing::error!("Failed to send serial payload: {err}");
                            break;
                        }
                        let (encoded_len, _) = incoming_buffer
                            .iter()
                            .enumerate()
                            .find(|(_idx, elem)| **elem == 0x00)
                            .expect("Need a terminator to have a valid COBS encoded payload");
                        incoming_buffer =
                            Vec::from_iter(incoming_buffer.into_iter().skip(encoded_len + 1));
                    }
                }
                tracing::info!("Exiting handler for incoming serial data");
            },
            async {
                while let Ok(msg) = from_simulator.recv().await {
                    let mut encoded_data = cobs::encode_vec(&msg[..]);
                    encoded_data.push(0x00);
                    tracing::debug!("Sending serial data: {encoded_data:02x?}");
                    // The following is hanging for unclear reasons...
                    if tokio::time::timeout(Duration::from_secs(5), async {
                        if let Err(err) = writer.write(&encoded_data[..]).await {
                            tracing::error!("Failed to write encoded serial packet: {err}");
                            return;
                        } else {
                            if let Err(err) = writer.flush().await {
                                tracing::error!("Failed to flush: {err}");
                            }
                        }
                    })
                    .await
                    .is_err()
                    {
                        tracing::debug!("Writing timed out");
                    } else {
                        tracing::debug!("Serial send complete");
                    }
                }
            }
        );
        tracing::info!("Exiting serial device listening context");
    }

    async fn handle_incoming_serial_bytes(
        mut reader: ReadHalf<File>,
        to_simulator: Sender<Vec<u8>>,
    ) {
        let mut incoming_buffer = Vec::with_capacity(1024);
        while let Ok(_bytes_read) = reader.read_buf(&mut incoming_buffer).await {
            if let Ok(decoded_data) = cobs::decode_vec(&incoming_buffer[..]) {
                tracing::debug!(
                    "Decoded a payload of {} bytes from buffer of {} bytes. Whole buffer: {incoming_buffer:02x?}",
                    decoded_data.len(),
                    incoming_buffer.len()
                );
                if let Err(err) = to_simulator.send(decoded_data).await {
                    tracing::error!("Failed to send serial payload: {err}");
                    break;
                }
                let (encoded_len, _) = incoming_buffer
                    .iter()
                    .enumerate()
                    .find(|(_idx, elem)| **elem == 0x00)
                    .expect("Need a terminator to have a valid COBS encoded payload");
                incoming_buffer = Vec::from_iter(incoming_buffer.into_iter().skip(encoded_len + 1));
            }
        }
        tracing::info!("Exiting handler for incoming serial data");
    }

    async fn handle_simulator_packet(
        mut writer: WriteHalf<File>,
        from_simulator: Receiver<Vec<u8>>,
    ) {
        while let Ok(msg) = from_simulator.recv().await {
            let mut encoded_data = cobs::encode_vec(&msg[..]);
            encoded_data.push(0x00);
            tracing::debug!("Sending serial data: {encoded_data:02x?}");
            // The following is hanging for unclear reasons...
            if tokio::time::timeout(Duration::from_secs(5), async {
                if let Err(err) = writer.write(&encoded_data[..]).await {
                    tracing::error!("Failed to write encoded serial packet: {err}");
                    return;
                } else {
                    if let Err(err) = writer.flush().await {
                        tracing::error!("Failed to flush: {err}");
                    }
                }
            })
            .await
            .is_err()
            {
                tracing::debug!("Writing timed out");
            } else {
                tracing::debug!("Serial send complete");
            }
        }
    }
}

pub async fn run(
    device_path: impl AsRef<Path>,
    to_simulator: Sender<Vec<u8>>,
    from_simulator: Receiver<Vec<u8>>,
) {
    let virtual_device = match VirtualSerial::new(device_path.as_ref()) {
        Ok(device) => device,
        Err(err) => {
            tracing::error!(
                "Unable to create virtual serial device {}: {err}",
                device_path.as_ref().display()
            );
            let _ = std::fs::remove_file(device_path.as_ref());
            return;
        }
    };

    virtual_device.listen(to_simulator, from_simulator).await
}
