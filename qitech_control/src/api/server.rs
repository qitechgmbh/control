use qitech_framework::MachineInstanceIdentification;
use qitech_framework_hub::Actor;
use qitech_framework_hub::ActorContext;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;

<<<<<<< HEAD
use crate::api::legacy::LegacyApiState;
use crate::api::legacy::LegacySharedState;
use crate::api::legacy::init_socket_io;
use crate::api::legacy::v1;
use crate::api::legacy::v2;
=======
use crate::api::legacy::LegacySharedState;
use crate::api::legacy::init_socket_io;
use crate::api::legacy::v1;
>>>>>>> 8e49141 (Jse control v2 (#1680))
use crate::api::types::SharedState;

pub struct Server {
    state: SharedState,
    state_legacy: LegacySharedState,
    machines_dirty_tx: mpsc::Sender<MachineInstanceIdentification>,
}

impl Server {
    pub fn new(
        state: SharedState,
        state_legacy: LegacySharedState,
        machines_dirty_tx: mpsc::Sender<MachineInstanceIdentification>,
    ) -> Self {
        Self {
            state,
            state_legacy,
            machines_dirty_tx,
        }
    }
}

impl Actor for Server {
    async fn run(self, ctx: ActorContext) {
        let router = axum::Router::new()
            .nest("/api/v1", v1::router())
<<<<<<< HEAD
            .nest("/api/v2", v2::router())
=======
            // .nest("/api/v2", v2::router())
>>>>>>> 8e49141 (Jse control v2 (#1680))
            // .nest("/api/v3", v3::router())
            .layer(init_socket_io(
                self.state.clone(),
                self.state_legacy.clone(),
            ))
            .layer(axum::Extension(self.state_legacy.clone()))
            .layer(CorsLayer::permissive())
<<<<<<< HEAD
            .with_state(LegacyApiState {
                ctx,
                machines_dirty_tx: self.machines_dirty_tx.clone(),
                legacy: self.state_legacy.clone(),
            });
=======
            .with_state((ctx, self.machines_dirty_tx.clone()));
>>>>>>> 8e49141 (Jse control v2 (#1680))

        //.nest("/api/v2", rest_api_router())
        //.layer(socketio_layer)
        //.layer(cors)
        //.layer(trace_layer);

        let listener = TcpListener::bind("0.0.0.0:3001")
            .await
            .expect("Failed to bind to port 3001");

        axum::serve(listener, router).await.unwrap();
    }
}
