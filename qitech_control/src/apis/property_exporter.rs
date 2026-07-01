use std::sync::Arc;
use machine_core::property::PropertyBatchExporter;
use tokio::{io::AsyncWriteExt, net::UnixStream, sync::mpsc};

use crate::app_state::{Properties, get_async_runtime};

pub fn start() -> mpsc::Sender<Arc<Properties>> {
    let rt = get_async_runtime();

    let stream = rt
        .block_on(async { UnixStream::connect("/tmp/qitech_ctrl_hub.sock").await })
        .expect("failed to connect to unix socket");

    let (tx, rx) = mpsc::channel(32);
    rt.spawn(task(stream, rx));
    
    tx
}

pub async fn task(
    mut stream: UnixStream,
    mut receiver: mpsc::Receiver<Arc<Properties>>,
) {
    loop {
        let Some(properties) = receiver.recv().await else {
            return;
        };

        let mut buf = [0u8; 16384];
        let data = serialize_properties(properties, &mut buf);

        // length prefix (little endian)
        let len = data.len() as u32;
        stream
            .write_all(&len.to_le_bytes())
            .await
            .expect("failed to write frame length");

        stream.write_all(data).await.expect("failed to write frame");

        stream.flush().await.expect("flush failed");
    }
}

fn serialize_properties(properties: Arc<Properties>, buf: &mut [u8]) -> &[u8] {
    let exporter = PropertyBatchExporter::new(true, &properties.float, &properties.int);
    postcard::to_slice(&exporter, buf).expect("serialization failed")
}
