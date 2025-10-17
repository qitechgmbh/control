# Machine Read API

The Machine Read API provides HTTP endpoints to query machine state and live data from external clients. This API is disabled by default and must be explicitly enabled via the frontend UI.

## Table of Contents

- [Overview](#overview)
- [Enabling the API](#enabling-the-api)
- [Authentication & Security](#authentication--security)
- [Machine Identification](#machine-identification)
- [API Endpoints](#api-endpoints)
- [Response Format](#response-format)
- [Code Examples](#code-examples)
- [Available Machines](#available-machines)
- [Error Handling](#error-handling)

## Overview

The Machine Read API allows external systems to:
- Query current machine state (configuration, modes, controller states)
- Read live sensor values (positions, speeds, measurements)
- Access both state and live data in a single snapshot

All endpoints are read-only. 

## Enabling the API

The API must be enabled before use. By default, it is **disabled** for security reasons.

Navigate to **Setup → Machines** in the Electron application and toggle the "Host Machine API" setting. The UI will display the IP addresses where the API is available once enabled.

## Authentication & Security

⚠️ **Important Security Considerations:**

- The API has **no authentication** by default
- It is exposed on all network interfaces when enabled
- Only enable on trusted networks
- Consider implementing firewall rules to restrict access
- The API is read-only, but exposes operational data

## Machine Identification

Machines are identified using a flexible addressing scheme:

### Identifier Formats

1. **Known Slugs** (recommended):
   - `winder2` → Winder V1 (vendor: 1, machine: 2)
   - `extruder2` → Extruder V1 (vendor: 1, machine: 4)
   - `laser1` → Laser V1 (vendor: 1, machine: 6)
   - `buffer1` → Buffer V1 (vendor: 1, machine: 8)
   - `aquapath1` → Aquapath V1 (vendor: 1, machine: 9)
   - `mock1` → Mock Machine (vendor: 1, machine: 7)

2. **Vendor-Machine Format**:
   - `1-2`, `1:2`, `1.2`, or `1_2` (all valid separators)
   - Format: `<vendor_id><separator><machine_id>`

3. **With Serial Number**:
   - `winder2-42` → Winder with serial 42
   - `1-2-42` → Same, using vendor-machine format
   - Separators: `-`, `:`, `.`, `_`

### Serial Number Handling

- If only one machine of a type exists, the serial is optional
- If multiple machines exist, you must specify the serial:
  - In the URL path: `/api/winder2/42/state`
  - Or let it be inferred if unique

**Automatic Serial Inference:**
If you don't specify a serial and only one machine of that type is connected, the API automatically uses it.

## API Endpoints

Base URL: `http://<host>:3000`

### 1. Get Machine State

Returns the most recent state event containing machine configuration and controller states.

```
GET /api/{identifier}/state
GET /api/{identifier}/{serial}/state
```

**Example:**
```bash
curl http://localhost:3000/api/winder2/state
curl http://localhost:3000/api/winder2/42/state
```

### 2. Get Live Values

Returns the most recent live values event containing sensor readings.

```
GET /api/{identifier}/live
GET /api/{identifier}/{serial}/live
```

**Example:**
```bash
curl http://localhost:3000/api/laser1/live
curl http://localhost:3000/api/1-6/100/live
```

### 3. Get Complete Snapshot

Returns both state and live values in a single response.

```
GET /api/{identifier}
GET /api/{identifier}/{serial}
```

**Example:**
```bash
curl http://localhost:3000/api/extruder2
curl http://localhost:3000/api/buffer1/1
```

## Response Format

All successful responses follow this structure:

```json
{
  "machine_identification_unique": {
    "machine_identification": {
      "vendor": 1,
      "machine": 2
    },
    "serial": 42
  },
  "state": {
    "name": "StateEvent",
    "ts": 1729177234567,
    "data": {
      // Machine-specific state data
    }
  },
  "live": {
    "name": "LiveValuesEvent",
    "ts": 1729177234890,
    "data": {
      // Machine-specific live values
    }
  }
}
```

### Fields

- `machine_identification_unique`: Full machine identification
  - `vendor`: Vendor ID (u16)
  - `machine`: Machine type ID (u16)
  - `serial`: Machine serial number (u16)
- `state`: State event (omitted if requesting only live values)
  - `name`: Always "StateEvent"
  - `ts`: Unix timestamp in milliseconds
  - `data`: Machine-specific state object
- `live`: Live values event (omitted if requesting only state)
  - `name`: Always "LiveValuesEvent"
  - `ts`: Unix timestamp in milliseconds
  - `data`: Machine-specific live values object

## Code Examples

### Python

```python
import requests
import json

# Get winder state
response = requests.get('http://192.168.1.100:3000/api/winder2/state')
if response.status_code == 200:
    data = response.json()
    state = data['state']['data']
    print(f"Puller state: {state['puller_state']}")
    print(f"Traverse state: {state['traverse_state']}")
else:
    print(f"Error: {response.status_code}")

# Get laser live values
response = requests.get('http://192.168.1.100:3000/api/laser1/live')
if response.status_code == 200:
    data = response.json()
    live = data['live']['data']
    print(f"Diameter: {live['diameter']} mm")
    print(f"X diameter: {live.get('x_diameter', 'N/A')} mm")
else:
    print(f"Error: {response.status_code}")

# Get complete snapshot
response = requests.get('http://192.168.1.100:3000/api/extruder2')
snapshot = response.json()
print(json.dumps(snapshot, indent=2))
```

### JavaScript/Node.js

```javascript
const axios = require('axios');

const BASE_URL = 'http://192.168.1.100:3000';

async function getWinderState() {
  try {
    const response = await axios.get(`${BASE_URL}/api/winder2/state`);
    const { state } = response.data;
    
    console.log('Timestamp:', new Date(state.ts));
    console.log('Mode:', state.data.mode_state);
    console.log('Puller:', state.data.puller_state);
    
    return state.data;
  } catch (error) {
    if (error.response?.status === 404) {
      console.error('Machine not found or API disabled');
    } else {
      console.error('Error:', error.message);
    }
  }
}

async function pollLaserDiameter() {
  try {
    const response = await axios.get(`${BASE_URL}/api/laser1/live`);
    const { live } = response.data;
    
    return {
      diameter: live.data.diameter,
      xDiameter: live.data.x_diameter,
      yDiameter: live.data.y_diameter,
      roundness: live.data.roundness,
      timestamp: live.ts
    };
  } catch (error) {
    console.error('Failed to get laser data:', error.message);
    return null;
  }
}

// Poll every 100ms
setInterval(async () => {
  const data = await pollLaserDiameter();
  if (data) {
    console.log(`Diameter: ${data.diameter.toFixed(3)} mm`);
  }
}, 100);
```

### Rust

```rust
use serde::{Deserialize, Serialize};
use reqwest;
use anyhow::Result;

#[derive(Debug, Deserialize)]
struct MachineSnapshot {
    machine_identification_unique: MachineIdentificationUnique,
    state: Option<EventPayload>,
    live: Option<EventPayload>,
}

#[derive(Debug, Deserialize)]
struct MachineIdentificationUnique {
    machine_identification: MachineIdentification,
    serial: u16,
}

#[derive(Debug, Deserialize)]
struct MachineIdentification {
    vendor: u16,
    machine: u16,
}

#[derive(Debug, Deserialize)]
struct EventPayload {
    name: String,
    ts: u64,
    data: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = reqwest::Client::new();
    
    // Get winder snapshot
    let snapshot: MachineSnapshot = client
        .get("http://192.168.1.100:3000/api/winder2")
        .send()
        .await?
        .json()
        .await?;
    
    println!("Machine: {}/{}/{}", 
        snapshot.machine_identification_unique.machine_identification.vendor,
        snapshot.machine_identification_unique.machine_identification.machine,
        snapshot.machine_identification_unique.serial
    );
    
    if let Some(state) = snapshot.state {
        println!("State at {}: {:?}", state.ts, state.data);
    }
    
    if let Some(live) = snapshot.live {
        println!("Live at {}: {:?}", live.ts, live.data);
    }
    
    Ok(())
}
```

### curl

```bash
# Get state (pretty-printed)
curl -s http://localhost:3000/api/winder2/state | jq '.'

# Get live values with specific serial
curl -s http://localhost:3000/api/laser1/42/live | jq '.live.data'

# Get snapshot and extract specific field
curl -s http://localhost:3000/api/extruder2 | \
  jq '.state.data.puller_state'

# Monitor live diameter in a loop
while true; do
  curl -s http://localhost:3000/api/laser1/live | \
    jq -r '.live.data.diameter'
  sleep 0.1
done
```

## Available Machines

Each machine type provides different state and live value structures:

### Winder V2 (`winder2`)

**State Event Fields:**
- `is_default_state`: bool
- `traverse_state`: Traverse controller state
- `puller_state`: Puller controller state
- `spool_automatic_action_state`: Spool automation state
- `mode_state`: Current operational mode
- `tension_arm_state`: Tension arm controller state
- `spool_speed_controller_state`: Spool speed control state
- `connected_machine_state`: Cross-connection state

**Live Values:**
- `traverse_position`: Position in mm (optional)
- `puller_speed`: Speed in m/min
- `spool_rpm`: Spool rotations per minute
- `tension_arm_angle`: Angle in degrees
- `spool_progress`: Pulled filament distance in meters

### Laser V1 (`laser1`)

**State Event Fields:**
- `is_default_state`: bool
- `laser_state`: Laser configuration and tolerances

**Live Values:**
- `diameter`: Measured diameter in mm
- `x_diameter`: X-axis diameter in mm (optional)
- `y_diameter`: Y-axis diameter in mm (optional)
- `roundness`: Roundness metric (optional)

### Extruder V2 (`extruder2`)

**State Event Fields:**
- `is_default_state`: bool
- `puller_state`: Puller controller state
- Various temperature and control states

**Live Values:**
- `puller_speed`: Speed in m/min
- Temperature readings
- Pressure values

### Buffer V1 (`buffer1`)

**State Event Fields:**
- `mode_state`: Buffer operational mode
- `connected_machine_state`: Cross-connection state

**Live Values:**
- Buffer-specific sensor readings

### Aquapath V1 (`aquapath1`)

**State Event Fields:**
- Aquapath-specific configuration

**Live Values:**
- Water cooling metrics
- Flow rates

### Mock Machine (`mock1`)

Used for testing and development. Provides sample data structures.

## Error Handling

### HTTP Status Codes

- `200 OK`: Successful request
- `404 Not Found`: Machine not found, API disabled, or no data available
- `500 Internal Server Error`: Server error

### Error Response Format

```json
{
  "error": "Error message describing the issue"
}
```

### Common Errors

1. **API Disabled**
   ```
   GET /api/winder2/state → 404
   Error: "Machine read API is disabled"
   ```
   **Solution:** Enable the API first

2. **Machine Not Found**
   ```
   GET /api/winder2/state → 404
   Error: "Machine 1/2/0 not found"
   ```
   **Solution:** Check machine is connected and identifier is correct

3. **Multiple Machines Without Serial**
   ```
   GET /api/winder2/state → 404
   Error: "Multiple machines found for vendor 1 machine 2. Specify the serial..."
   ```
   **Solution:** Add serial to URL: `/api/winder2/42/state`

4. **No Data Available**
   ```
   GET /api/laser1/state → 404
   Error: "No state event available for machine 1/6/1"
   ```
   **Solution:** Wait for machine to publish events or check machine is running

5. **Machine Disconnected**
   ```
   GET /api/extruder2/live → 500
   Error: "Machine 1/4/1 is disconnected"
   ```
   **Solution:** Check machine connection and hardware

6. **Invalid Identifier**
   ```
   GET /api/unknown/state → 404
   Error: "Unable to parse machine identifier 'unknown'..."
   ```
   **Solution:** Use a valid identifier or vendor-machine format

## Implementation Notes

### Data Freshness

- Events are cached by the machine namespace
- State events are typically updated when configuration changes
- Live values are updated periodically (typically 10-100ms depending on machine)
- The `ts` field indicates when the event was generated (Unix milliseconds)

### Performance Considerations

- The API returns cached events, so there's minimal overhead
- Polling at 10Hz (100ms) is generally safe for live values
- State values change less frequently and don't need rapid polling
- Use snapshots (`/api/{identifier}`) to reduce request count

### Socket.io Alternative

For real-time updates, consider using Socket.io instead:
- Namespace: `/machine/{vendor}/{machine}/{serial}`
- Events: `StateEvent` and `LiveValuesEvent`
- Provides push-based updates rather than polling

See the Socket.io documentation for details.

## Related Documentation

- [Architecture Overview](../architecture-overview.md)
- [Adding a Machine](./adding-a-machine.md)
- [Control Loop](../control-loop.md)
