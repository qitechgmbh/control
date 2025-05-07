# Threads & Async
We utilize different threads and mix `smol` with `tokio`.
Mid-term we want to get rid of `tokio` for the ethercat part but keep it for `axum` (REST) and `socketioxide` (SocketIO).

## Threads
### Main Thread from `server::main::main`

Main thread that inits the program and starts the other threads.

Listend to a channel for panics other threads can send via the `send_panic` function.
If the main thread recieves the panic signal it exits (`exit(1)`) the program.

### `ApiThread` from `server::api::init::init_api`
Creates a tokio runtime for the `init_api` function which starts the `axum` & `socketoxide` servers.

Used `send_panic` to exit program if this thread panics.

### Ethercat Interface Test Threads from `server::ethercat::init::init_ethercat`  
Starts threads for testing different interfaces in parallel.

Uses Smol `LocalExecutor`.

### `LoopThread` from `server::ethercat::init::init_ethercat`
Uses thread-local `smol` runtime.

Runs the main control loop.

Used `send_panic` to exit program if this thread panics.

TODO: Should use realtime priority.

### `EthercatTxRxThread` from `server::ethercat::loop::setup_loop`
Runs the Ethercrab TX/RX loop.

TODO: Should use realtime priority.

Used `send_panic` to exit program if this thread panics.