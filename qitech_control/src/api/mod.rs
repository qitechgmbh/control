mod socketio;

use qitech_framework_hub::Module;
use qitech_framework_hub::ModuleContext;

struct ApiModule;

impl Module for ApiModule {
    async fn run(self, mut ctx: ModuleContext) {
        loop {
            let report = ctx.report_rx.recv().await.unwrap();
            println!("Received report");
        }
    }
}

/// Module for dispatching socket io events.
/// Scans reports and detects if a machine config or state event occured
struct SocketIODispatcherModule {}

impl Module for SocketIODispatcherModule {
    async fn run(self, mut ctx: ModuleContext) {
        loop {
            let report = ctx.report_rx.recv().await.unwrap();
            println!("Received report");
        }
    }
}
