# REST API `/api/v2`

The Qitech Control Panel exposes a small HTTP interface for discovering machines and reading their current values. All examples below assume you are connected to the panel’s Ethernet subnet and talk directly to the panel at:

- **Base URL:** `https://IP_ADDRESS`

> **Schema note:** This documentation intentionally stays light on field details beyond what is shown in the examples. For the exact datatypes and complete payload shapes, refer to the corresponding Rust types (linked below).

---

## Authentication

The panel performs Basic Auth authentication. 
For EVERY request made over HTTPS Basic Auth is expected.

To set the password on the Panel for Basic Auth do the following:
1. Open a Terminal
2. cd /home/qitech/control
3. ./credentials.sh

credentials.sh prints the username and password for the api ONCE in the console. 
Afterwards its accessible in machine_password in /tmp/ until the machine shuts down.
If you ever forget your password you can simply run ./credentials.sh again, resetting the password used, however the username will always be machine 

For HTTPS+Authentication we have Caddy (a reverse proxy, with automatic https) in front of the internal API.
The panel is configured like any other NetworkManager linux and is able to either receive an IP-Address over DHCP or a statically defined one.
Where the static IP is most likely the easiest to work with.
In theory the HTTPS-Api Can be accessed over any network interface that is configured by NetworkManager.

If DNS is available on the subnet, you may be able to resolve `qitech.control`.

## Handling Self signed Certificate Warnings

What you might notice is that caddy automatically generates certificates with its internal CA.
These are untrusted by default in browsers,tools like curl and https libraries.

For most programs you can either ignore this warning or you can manually trust the certificate on your system:
```
./.local/share/caddy/pki/authorities/local/root.crt
/var/lib/caddy/.local>/share/caddy/pki/authorities/local/root.crt
```

## Bypassing HTTPS and Authentication
If you have an isolated network and dont wish to perform Authentication, then you can achieve this by: 
1. Forking our repository
2. Changing the Open Ports in nixos/os/configuration.nix:
```
  # All the way at the Bottom   
  networking.firewall.allowedUDPPorts = [ 53 67 69 ];
  # Change this 
  networking.firewall.allowedTCPPorts = [ 443 ];
  # To this
  networking.firewall.allowedTCPPorts = [ 3001 ];
```

Again, you should only ever do this if you have your own reverse proxy with authentication or you have an isolated network.


---

## List machines `GET /api/v2/machine`

Returns the set of machines currently known/connected to the panel.

Machines are identified by:

- `slug`: the machine type / model identifier (string)
- `serial`: the specific machine instance identifier (int)

Each machine also includes a `legacy_id` to support older **v1** workflows. If a machine reports an issue, the `error` field may be present and non-null (containing an error message).

### Example request

```bash
curl -X GET "https://IP_ADDRESS/api/v2/machine"
```

### Example response

```json
{
  "machines": [
    {
      "legacy_id": {
        "machine_identification": {
          "vendor": 1,
          "machine": 7
        },
        "serial": 57922
      },
      "serial": 57922,
      "vendor": "QiTech",
      "slug": "mock",
      "error": null
    },
    {
      "legacy_id": {
        "machine_identification": {
          "vendor": 1,
          "machine": 4
        },
        "serial": 57922
      },
      "serial": 57922,
      "vendor": "QiTech",
      "slug": "extruder_v1",
      "error": null
    },
    {
      "legacy_id": {
        "machine_identification": {
          "vendor": 1,
          "machine": 2
        },
        "serial": 57922
      },
      "serial": 57922,
      "vendor": "QiTech",
      "slug": "winder_v1",
      "error": null
    },
    {
      "legacy_id": {
        "machine_identification": {
          "vendor": 1,
          "machine": 10
        },
        "serial": 48879
      },
      "serial": 48879,
      "vendor": "QiTech",
      "slug": "wago_power_v1",
      "error": null
    }
  ]
}
```

---

## Get current values `GET /api/v2/machine/<slug>/<serial>`

Returns all currently known values for a single machine.

Values are categorized into two groups:

- **State**: requested/commanded values (these typically change only after a state-change request, or if another controller updates them)
- **Live Values**: measured/observed values coming from the machine and potentially changing quickly

This REST endpoint returns **only the current snapshot**, not a stream of live values.
To receive continuous updates (via WebSockets), subscribe to the machine namespace (see **WebSockets** below).

### Example request (mock machine)

```bash
curl -X GET "https://IP_ADDRESS/api/v2/machine/mock/57922"
```

### Example response

```json
{
  "machine": {
    "legacy_id": {
      "machine_identification": {
        "vendor": 1,
        "machine": 7
      },
      "serial": 57922
    },
    "serial": 57922,
    "vendor": "QiTech",
    "slug": "mock",
    "error": null
  },
  "state": {
    "frequency1": 100.0,
    "frequency2": 200.0,
    "frequency3": 500.0,
    "is_default_state": false,
    "mode_state": {
      "mode": "Running"
    }
  },
  "live_values": {
    "amplitude1": -0.03438523433309566,
    "amplitude2": -0.06872980145477608,
    "amplitude3": -0.1711138370170743,
    "amplitude_sum": -0.27422887280494607
  }
}
```

---

## Change machine state `POST /api/v1/machine/<slug>/<serial>`

State changes are submitted as **mutations**. The mutation payload is defined per machine type in Rust. Conceptually, each item in the mutation list represents a setter-style operation that is applied by the real-time control loop.

The API does **not** return the newly-applied state in the POST response. The panel runs a real-time loop and generally won’t block waiting for the physical system to converge. Instead:

- Submit the mutation via `POST`
- Poll `GET /api/v2/machine/<slug>/<serial>` to observe the updated state and/or any reported errors

### Example request (mock machine)

```bash
curl -X POST \
  -d  \
  -H "Content-Type: application/json" \
  "http://IP_ADDRESS/api/v1/machine/mock/57922"
```

### Example response

```json
null
```

---

## WebSockets

For continuous updates, subscribe to a machine-specific namespace derived from its `legacy_id`:

- Namespace: `/machine/<vendor>/<id>/<serial>`

The stream emits events for:

- state changes (`StateEvent`)
- live value updates (`LiveValuesEvent`)

Both event payloads use the same machine-specific schema as the `/api/v2` REST responses.

---

## List of all machines

Below is a template you can fill with links to the relevant Rust types (mutations + state/live structs).
For each machine, link to:

- **Mutations:** the request payload type used by `POST /api/v1/machine/<slug>/<serial>`
- **State / Live Values:** the response payload types returned by `GET /api/v2/machine/<slug>/<serial>`

### Machines

- **winder_v1**

  - Mutations: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/winder2/api.rs#L87>
  - State: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/winder2/api.rs#L163>
  - Live Values: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/winder2/api.rs#L143>

- **extruder_v1**

  - Mutations: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/extruder1/api.rs#L214>
  - State: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/extruder1/api.rs#L89>
  - Live Values: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/extruder1/api.rs#L55>

- **laser_v1**

  - Mutations: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/laser/api.rs#L88>
  - State: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/laser/api.rs#L32>
  - Live Values: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/laser/api.rs#L17>

- **mock**

  - Mutations: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/mock/api.rs#L84>
  - State: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/mock/api.rs#L38>
  - Live Values: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/mock/api.rs#L24>

- **extruder_v2**

  - Mutations: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/extruder2/api.rs#L126>
  - State: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/extruder2/api.rs#L93>
  - Live Values: <https://github.com/qitechgmbh/control/blob/1ec20074e9030a0ed1739ca9d9a77e298a2652a3/machines/src/extruder2/api.rs#L59>
