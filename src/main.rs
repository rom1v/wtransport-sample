use log::error;
use std::future::Future;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::task::{JoinError, JoinHandle};
use tokio_util::sync::CancellationToken;
use wtransport;
use wtransport::tls::Certificate;
use wtransport::Endpoint;
use wtransport::ServerConfig;

#[derive(Error, Debug)]
enum SampleError {
    #[error("Connection error")]
    ConnectionError(#[from] wtransport::error::ConnectionError),
    #[error("Stream opening error")]
    StreamOpeningError(#[from] wtransport::error::StreamOpeningError),
    #[error("Stream read error")]
    StreamReadError(#[from] wtransport::error::StreamReadError),
    #[error("Stream write error")]
    StreamWriteError(#[from] wtransport::error::StreamWriteError),
    #[error("Send datagram error")]
    SendDatagramError(#[from] wtransport::error::SendDatagramError),
    #[error("Join error")]
    JoinError(#[from] JoinError),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("Generic error: {0}")]
    Generic(String),
}

fn spawn_task<F>(task_name: String, token: CancellationToken, task: F) -> JoinHandle<()>
    where F: Future<Output = Result<(), SampleError>> + Send + 'static {
    tokio::spawn(async move {
        tokio::select! {
            _ = token.cancelled() => {}
            ret = task => {
                if let Err(err) = ret {
                    error!("{task_name} failed: {err:?}");
                    // cancel all tasks on any error
                    token.cancel();
                }
            }
        }
    })
}

#[tokio::main]
async fn main() -> Result<(), SampleError> {
    let config = ServerConfig::builder()
        .with_bind_address(SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 4433))
        .with_certificate(Certificate::load("cert.pem", "key.pem")?)
        .keep_alive_interval(Some(Duration::from_secs(3)))
        .build();

    let server = Endpoint::server(config)?;

    loop {
        println!("Waiting for incoming connection...");
        let connecting = server.accept().await.ok_or(SampleError::Generic(
            "No Connecting instance returned".to_string(),
        ))?;
        let connection = connecting.await?;
        println!("Waiting for data from client...");

        // wtransport Connection is not clonable
        let connection = Arc::new(connection);

        let token = CancellationToken::new();

        let conn = connection.clone();
        let bi = spawn_task("accept_bi".to_string(), token.clone(), async move {
            let mut buffer = vec![0; 4096];
            loop {
                let (mut send, mut recv) = conn.accept_bi().await?;
                let bytes_read = recv.read(&mut buffer).await?.unwrap_or(0);

                println!("Received {bytes_read} bytes on bi-stream");
                let response = bytes_read.to_string();
                send.write_all(&response.as_bytes()).await?;
            }
        });
        let conn = connection.clone();
        let uni = spawn_task("accept_uni".to_string(), token.clone(), async move {
            let mut buffer = vec![0; 4096];
            loop {
                let mut recv = conn.accept_uni().await?;
                let bytes_read = recv.read(&mut buffer).await?.unwrap_or(0);
                println!("Received {bytes_read} bytes on uni-stream");

                // DOES NOT COMPILE: OpeningUniStream is not Send
                //let response = bytes_read.to_string();
                //let send = conn.open_uni().await?.await?;
                //send.write_all(&response.as_bytes()).await?;
            }
        });
        let conn = connection.clone();
        let dg = spawn_task("receive_datagram".to_string(), token.clone(), async move {
            loop {
                let datagram = conn.receive_datagram().await?;
                println!("Received datagram of {} bytes", datagram.len());

                let response = datagram.len().to_string();
                conn.send_datagram(&response.as_bytes())?;
            }
        });

        bi.await?;
        uni.await?;
        dg.await?;
    }
}
