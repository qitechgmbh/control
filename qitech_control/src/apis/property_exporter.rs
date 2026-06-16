use std::sync::Arc;

use property::{DirtyPropertySetExportView, PropertySet};
use tokio::{io::AsyncWriteExt, net::UnixStream, sync::mpsc};

use crate::app_state::get_async_runtime;

pub fn start() -> mpsc::Sender<Arc<PropertySet>> {
    let rt = get_async_runtime();

    let (tx, rx) = mpsc::channel(32);

    let stream = rt
        .block_on(async { UnixStream::connect("/tmp/qitech_ctrl_properties.sock").await })
        .expect("failed to connect to unix socket");

    rt.spawn(async move {
        task(stream, rx).await;
    });

    tx
}

pub async fn task(
    mut stream: UnixStream,
    mut receiver: mpsc::Receiver<Arc<PropertySet>>,
) {
    loop {
        let Some(properties) = receiver.recv().await else {
            return;
        };

        let mut buf = [0u8; 16384];
        let data = serialize_properties(properties, &mut buf);

        // magic
        let magic: u32 = 0xB00B135;
        stream
            .write_all(&magic.to_le_bytes())
            .await
            .expect("failed to write frame length");

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

fn serialize_properties(properties: Arc<property::PropertySet>, buf: &mut [u8]) -> &[u8] {
    let export_view = DirtyPropertySetExportView::from(properties.as_ref());
    postcard::to_slice(&export_view, buf).expect("serialization failed")
}
