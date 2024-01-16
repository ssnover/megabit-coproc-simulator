#[cfg(feature = "ssr")]
mod backend;

#[cfg(feature = "csr")]
mod frontend;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use backend::*;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "megabit_coproc_simulator=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (from_ws_tx, from_ws_rx) = async_channel::unbounded();
    let (to_ws_tx, to_ws_rx) = async_channel::unbounded();
    let (from_serial_tx, from_serial_rx) = async_channel::unbounded();
    let (to_serial_tx, to_serial_rx) = async_channel::unbounded();

    tokio::select! {
        _ = web_server::serve(8000, to_ws_rx, from_ws_tx) => {
            tracing::error!("HTTP server exited");
        },
        _ = simulator::run(from_ws_rx, from_serial_rx, to_ws_tx, to_serial_tx) => {
            tracing::error!("Simulator exited");
        }
        _ = serial::run("/dev/megabit-sim", from_serial_tx, to_serial_rx) => {
            tracing::error!("Virtual TTY exited");
        }
    };
}

#[cfg(feature = "csr")]
fn main() {
    yew::Renderer::<frontend::App>::new().render();
}
