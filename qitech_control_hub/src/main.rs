use std::sync::Arc;
mod database_exporter;
use clickhouse::Client;
use tokio::{io::AsyncReadExt, net::{UnixListener, UnixStream}, select, signal::unix::{Signal, SignalKind, signal}, sync::broadcast};
use property::ExportedPropertySet;

#[tokio::main]
async fn main() {
    let socket_path = "/tmp/qitech_ctrl_hub.sock";

    std::fs::remove_file(socket_path).expect("msg");
    let listener = UnixListener::bind(socket_path).unwrap();
    let mut sigabrt = signal(SignalKind::terminate()).expect("Needs hook");

    let (mut tx, rx) = broadcast::channel(512);

    let client = Client::default()
        .with_url("http://localhost:8123")
        .with_user("default")
        // .with_password("")
        .with_database("qitech_ctrl");

    tokio::spawn(database_exporter::run(client, rx));

    loop {
        let stream = select! {
            biased;

            _ = sigabrt.recv() => {
                println!("Received SIGABRT");
                break;
            }

            res = listener.accept() => {
                let (stream, _) = match res {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Failed to accept connection: {e}");
                        continue;
                    }
                };

                stream
            }
        };

        handle_client(&mut sigabrt, stream, &mut tx).await;
    }
}

pub async fn handle_client(
    sigabrt: &mut Signal, 
    mut stream: UnixStream,
    tx: &mut broadcast::Sender<Arc<ExportedPropertySet>>,
) {
    loop {
        let len = select! {
            biased;

            _ = sigabrt.recv() => {
                println!("Received SIGABRT");
                break;
            }

            result = stream.read_u32_le() => {
                match result {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Failed to read len: {e}");
                        return;
                    },
                }
            }
        };

        let mut buf = vec![0u8; len as usize];
        select! {
            biased;

            _ = sigabrt.recv() => {
                println!("Received SIGABRT");
                break;
            }

            result = stream.read_exact(&mut buf) => {
                if let Err(e) = result {
                    eprintln!("Failed to read data: {e}");
                    continue;
                }
            }
        };

        let snapshot: ExportedPropertySet = postcard::from_bytes(&buf)
            .expect("REMOVE LATER");
    }
}