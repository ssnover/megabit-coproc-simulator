use async_channel::{Receiver, Sender};
use nix::pty::{self, OpenptyResult};
use std::{
    io,
    ops::ControlFlow,
    os::fd::{AsRawFd, FromRawFd},
    path::{Path, PathBuf},
};
use tokio::{
    fs::File,
    io::{ReadHalf, WriteHalf},
};

pub struct VirtualSerial {
    host_file: File,
    _host_path: PathBuf,
    _device_file: File,
    device_path: PathBuf,
}

impl VirtualSerial {
    pub fn new(symlink_path: impl AsRef<Path>) -> io::Result<Self> {
        let OpenptyResult { master, slave } = pty::openpty(None, None).unwrap();
        let serial = Self {
            host_file: unsafe { File::from_raw_fd(master.as_raw_fd()) },
            _host_path: std::fs::read_link(format!("/proc/self/fd/{}", master.as_raw_fd()))?,
            _device_file: unsafe { File::from_raw_fd(slave.as_raw_fd()) },
            device_path: std::fs::read_link(format!("/proc/self/fd/{}", slave.as_raw_fd()))?,
        };
        if symlink_path.as_ref().exists() {
            std::fs::remove_file(symlink_path.as_ref())?;
        }
        std::os::unix::fs::symlink(&serial.device_path, symlink_path)?;
        Ok(serial)
    }

    pub async fn listen(self, to_simulator: Sender<Vec<u8>>, from_simulator: Receiver<Vec<u8>>) {
        let (reader, writer) = tokio::io::split(self.host_file);

        tokio::join!(
            Self::handle_incoming_serial_bytes(reader, to_simulator),
            Self::handle_incoming_simulator_packet(writer, from_simulator)
        );
    }

    async fn handle_incoming_serial_bytes(
        reader: ReadHalf<File>,
        to_simulator: Sender<Vec<u8>>,
    ) -> ControlFlow<(), ()> {
        todo!()
    }

    async fn handle_incoming_simulator_packet(
        writer: WriteHalf<File>,
        from_simulator: Receiver<Vec<u8>>,
    ) -> ControlFlow<(), ()> {
        todo!()
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
