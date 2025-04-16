# Threads & Async
We utilize different threads and mix `smol` with `tokio`.
Mid-term we want to get rid of `tokio` for the ethercat part but keep it for `axum` (REST) and `socketioxide` (SocketIO).

## Threads
### Main Thread from `server::main::main`
starts the main thread.

Has Tokio runtime which can spawn more threads on it's own.


### Ethercat Interface Test Threads from `server::ethercat::init::init_ethercat`  
Starts threads for testing different interfaces in parallel.

Uses Smol `LocalExecutor`.

### `EthercatThread` from `server::ethercat::init::init_ethercat`
Uses thread-local `tokio` runtime.

Starts the Ethercrab TX/RX thread.

Runs the main control loop.

TODO: Should use realtime priority.

### Ethercrab TX/RX Thread from `server::ethercat::loop::setup_loop`
Runs the Ethercrab TX/RX loop.

TODO: Should use realtime priority.