# Threads & Async
We utilize different threads and mix `smol` with `tokio`.
Mid-term we want to get rid of `tokio` for the ethercat part but keep it for `axum` (REST) and `socketioxide` (SocketIO).

## Threads
### Main Thread from `server::main::main`

Creates to tokio runtime for the `init_api` funciton which starts the `axum` & `socketoxide` servers.

### Ethercat Interface Test Threads from `server::ethercat::init::init_ethercat`  
Starts threads for testing different interfaces in parallel.

Uses Smol `LocalExecutor`.

### `EthercatSetupLoopThread` from `server::ethercat::init::init_ethercat`
Uses thread-local `smol` runtime.

Starts the Ethercrab TX/RX thread.

Runs the main control loop.

TODO: Should use realtime priority.

### Ethercrab TX/RX Thread from `server::ethercat::loop::setup_loop`
Runs the Ethercrab TX/RX loop.

TODO: Should use realtime priority.