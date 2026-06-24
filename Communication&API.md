# Communication & API

The operator interface reaches the backend over two channels, both served by the same HTTP server on port `3001`: a **REST API** for commands and reads, and **Socket.IO** for live data. Commands go down through REST; state and live values come up through Socket.IO.

## REST API

The REST API is per-machine:

| Method & path | Purpose |
|---|---|
| `GET /machine` | List the connected machines |
| `GET /machine/{slug}/{serial}` | Read one machine |
| `POST /machine/{slug}/{serial}` | Send a command to one machine |

`{slug}` is the machine's short name (derived from its identification) and `{serial}` is its serial number. A `POST` carries a JSON body, the server forwards it to the target machine as a message (`HttpApiJsonRequest`), and the machine applies it on its next cycle through its `api_mutate` handler, so commands never block the real-time loop. The server binds `0.0.0.0:3001`.

## Socket.IO

Live data flows over Socket.IO, encoded with [MessagePack](https://msgpack.org/) for compactness. There are two kinds of namespace:

- A **main namespace** carries cross-cutting events: the list of connected machines, the EtherCAT devices, and network-interface discovery.
- A **per-machine namespace** carries that machine's state and live values, emitted as typed events. It is keyed by the machine identification — `/machine/{vendor}/{machine}/{serial}` — the same key the frontend subscribes to.

## Event caching

Each event type declares a caching strategy (for example, keep the latest value). The namespace retains those cached events, so a client that subscribes late immediately receives the machine's current state instead of waiting for the next update.

## Putting it together

To drive a machine, a client lists the machines (`GET /machine`, or the main namespace), subscribes to that machine's Socket.IO namespace to receive its live state, and `POST`s commands to `/machine/{slug}/{serial}`. See **Frontend** for how the desktop application does exactly this.
