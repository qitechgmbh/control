

// /tmp/qitech_ctrl_mutation.sock
// /tmp/qitech_ctrl_property.sock

use axum::{Router, routing::get};
use tokio::net::UnixListener;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;

async fn handler() -> &'static str {
    "hello"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));

    let listener = UnixListener::bind("/tmp/my.sock").unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);

        let app = app.clone();

        tokio::spawn(async move {
            http1::Builder::new()
                .serve_connection(io, app)
                .await
                .unwrap();
        });
    }
}