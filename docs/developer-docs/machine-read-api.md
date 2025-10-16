# Machine Read API

The server exposes HTTP endpoints that return the latest machine state and live values. These routes are bound to `0.0.0.0:3001`, so any host that can reach the control server over the network can call them. The API is enabled by default and can be toggled from the UI or through a dedicated configuration endpoint.

## Enabling or disabling the API

- **UI:** open *Setup → Machines* and use the *Host Machine API* toggle.
- **REST:**
  - `GET /api/v1/machine/api/enabled` → `{ "enabled": true }`
  - `POST /api/v1/machine/api/enabled` with `{ "enabled": false }` to disable (or `true` to enable).

When the API is disabled, all read endpoints return `404`.

## Machine identification

Each machine is addressed through a short identifier. The server accepts the following forms:

| Identifier | Example | Notes |
| --- | --- | --- |
| Slug | `winder2` | Slugs map to known machine types. If multiple machines of the same type are connected you must add a serial. |
| Slug with serial suffix | `winder2-42` | Attach the serial using `-`, `_`, `.`, or `:`. |
| Slug with path serial | `winder2/42` | Supply the serial as an additional path segment (`/api/winder2/42/state`). |
| Vendor/Machine | `1-8` | Numeric vendor/machine IDs in decimal or hex (`0x01-0x08`). |
| Vendor/Machine with serial | `1-8-42` or `0x01-0x08-0x2A` | Same separators as above. |

If the resolved identifier matches multiple machines, the API returns an error asking for the serial number.

## Endpoints

| Route | Description |
| --- | --- |
| `GET /api/{identifier}/state` | Latest cached state event (`StateEvent`). |
| `GET /api/{identifier}/live` | Latest cached live values event (`LiveValuesEvent`). |
| `GET /api/{identifier}` | Aggregated snapshot containing both state and live events (when available). |

Responses share the same shape:

```json
{
  "machine_identification_unique": {
    "machine_identification": { "vendor": 1, "machine": 8 },
    "serial": 42
  },
  "state": {
    "name": "StateEvent",
    "ts": 1737908215123,
    "data": { "mode": "Standby" }
  },
  "live": {
    "name": "LiveValuesEvent",
    "ts": 1737908214932,
    "data": { "pressure": 4.8, "temperature": 185.2 }
  }
}
```

`state` and `live` are omitted when the requested resource is not included (for example, the `/state` route only returns `state`). Timestamps are milliseconds since Unix epoch.

## Examples

Fetch the latest Buffer state (vendor `1`, machine `8`, serial `42`):

```bash
curl http://SERVER_IP:3001/api/buffer1-42/state
```

Fetch the latest live event using the slug form:

```bash
curl http://SERVER_IP:3001/api/buffer1/live
```

If more than one Buffer machine is connected the call above returns an error asking for the serial. Provide it via a suffix or path segment:

```bash
curl http://SERVER_IP:3001/api/buffer1/42/live
```

Fetch a combined snapshot:

```bash
curl http://SERVER_IP:3001/api/1-8-42
```

Check whether the API is enabled:

```bash
curl http://SERVER_IP:3001/api/v1/machine/api/enabled
```

Disable read access:

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}' \
  http://SERVER_IP:3001/api/v1/machine/api/enabled
```

Re-enable access by posting `{ "enabled": true }`.
