# Read-Only API Documentation

The Read-Only API provides external applications with safe, read-only access to machine state and live sensor data. This API is disabled by default and must be explicitly enabled through the control panel.

## Table of Contents

- [Overview](#overview)
- [Security & Configuration](#security--configuration)
- [API Endpoint](#api-endpoint)
- [Request Format](#request-format)
- [Response Format](#response-format)
- [Event Types](#event-types)
- [Code Examples](#code-examples)
  - [Python Examples](#python-examples)
  - [JavaScript Examples](#javascript-examples)
- [Best Practices](#best-practices)

## Overview

The Read-Only API allows you to:
- Query machine state information (mode, settings, connected machines)
- Retrieve live sensor data (temperatures, pressures, speeds, etc.)
- Access specific event types or all available events
- Integrate with external monitoring, logging, or data analysis systems

**Important:** This API is read-only. Machine control and mutations must be performed through the standard WebSocket/REST interface from the authorized control panel.

## Security & Configuration

### Enabling the API

The Read-Only API must be enabled in the control panel:

1. Navigate to **Setup → Machines**
2. Toggle "Enable Read-Only API"
3. The API will be accessible on all network interfaces on port 3001

### Network Access

When enabled, the API is available at:
```
http://<machine-ip>:3001/api/v1/machine/event
```

The control panel will display an IP addresses where the API can be reached.

### Security Considerations

- The API provides unrestricted read access to machine data
- Enable only when needed and in trusted networks

## API Endpoint

**Endpoint:** `POST /api/v1/machine/event`

**Content-Type:** `application/json`

## Request Format

### Request Body Schema

```json
{
  "machine_identification_unique": {
    "machine_identification": {
      "vendor": <number>,
      "machine": <number>
    },
    "serial": <number>
  },
  "events": ["LiveValues", "State"]
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `machine_identification_unique` | Object | Yes | Unique identifier for the target machine |
| `machine_identification_unique.machine_identification.vendor` | Number | Yes | Vendor ID (e.g., 1 for QiTech) |
| `machine_identification_unique.machine_identification.machine` | Number | Yes | Machine type ID (see machine types below) |
| `machine_identification_unique.serial` | Number | Yes | Machine serial number (displayed in Setup → Machines) |
| `events` | Array<String> | Yes | Array of event type names to retrieve. Valid values: "LiveValues", "State" |

### Event Type Specification

The `events` field is a required array of event type names. Include the names of the event types you want to retrieve.

**Examples:**
- `"events": ["LiveValues", "State"]` → Returns both LiveValues and State with all fields
- `"events": ["LiveValues"]` → Returns only LiveValues with all fields
- `"events": ["State"]` → Returns only State with all fields
- `"events": []` → Returns no events (empty data object)

### Machine Types

Common machine type IDs:

| Machine Type | ID |
|--------------|-----|
| Winder V2 | 2 |
| Extruder V1/V2 | 4 |
| Laser V1 | 6 |
| Mock Machine | 7 |
| Buffer V1 | 8 |
| AquaPath V1 | 9 |

## Response Format

### Success Response Schema

```json
{
  "success": true,
  "error": null,
  "data": {
    "State": { /* state data */ },
    "LiveValues": { /* live values data */ }
  }
}
```

### Error Response Schema

```json
{
  "success": false,
  "error": "Error message",
  "data": null
}
```

### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `success` | Boolean | `true` if request succeeded, `false` otherwise |
| `error` | String \| null | Error message if `success` is `false` |
| `data` | Object \| null | Event data organized by event type names |

### Data Object Structure

The `data` object contains the requested events as top-level keys:

```json
{
  "State": {
    // Machine state data
    "mode_state": { "mode": "automatic" },
    "connected_machine_state": { /* ... */ },
    // ... other state fields
  },
  "LiveValues": {
    // Real-time sensor data
    "temperature": 180.5,
    "pressure": 25.3,
    "motor_rpm": 1450.0,
    // ... other live values
  }
}
```

**Note:** The exact structure of `State` and `LiveValues` varies by machine type. Each machine exposes different sensors and state information.

## Event Types

### State Event

Contains machine configuration and operational state:

**Common Fields (varies by machine):**
- `mode_state` - Current operational mode
- `connected_machine_state` - Information about connected upstream/downstream machines
- Configuration settings (temperatures, speeds, limits)
- PID controller settings
- Target values and setpoints

### LiveValues Event

Contains real-time sensor readings and calculated values:

**Common Fields (varies by machine):**
- Temperature readings (°C)
- Pressure readings (bar)
- Speed/velocity measurements (RPM, m/min)
- Position information (mm)
- Power consumption (W)
- Flow rates (L/min)
- Angles (degrees)

## Code Examples

### Python Examples

#### Basic Example - Get All Events

```python
import requests
import json

# Configuration
API_URL = "http://192.168.1.100:3001/api/v1/machine/event"

# Machine identification
machine_id = {
    "machine_identification_unique": {
        "machine_identification": {
            "vendor": 1,  # QiTech
            "machine": 4   # Extruder V1/V2
        },
        "serial": 4
    },
    "events": ["LiveValues", "State"]
}

# Request all events
response = requests.post(API_URL, json=machine_id)

if response.status_code == 200:
    data = response.json()
    if data["success"]:
        print("State:", json.dumps(data["data"]["State"], indent=2))
        print("LiveValues:", json.dumps(data["data"]["LiveValues"], indent=2))
    else:
        print("Error:", data["error"])
else:
    print(f"HTTP Error: {response.status_code}")
```

#### Get Specific Events

```python
import requests

API_URL = "http://192.168.1.100:3001/api/v1/machine/event"

# Request only LiveValues, exclude State
request_body = {
    "machine_identification_unique": {
        "machine_identification": {
            "vendor": 1,
            "machine": 2  # Winder V2
        },
        "serial": 2
    },
    "events": ["LiveValues"]
}

response = requests.post(API_URL, json=request_body)
data = response.json()

if data["success"]:
    live_values = data["data"]["LiveValues"]
    print(f"Puller Speed: {live_values['puller_speed']} m/min")
    print(f"Spool RPM: {live_values['spool_rpm']} RPM")
    print(f"Tension Arm Angle: {live_values['tension_arm_angle']}°")
```

#### Continuous Monitoring

```python
import requests
import time
from datetime import datetime

API_URL = "http://192.168.1.100:3001/api/v1/machine/event"

machine_id = {
    "machine_identification_unique": {
        "machine_identification": {
            "vendor": 1,
            "machine": 4  # Extruder V1/V2
        },
        "serial": 4
    },
    "events": ["LiveValues"]
}

def monitor_temperature():
    """Monitor extruder temperature every 5 seconds"""
    while True:
        try:
            response = requests.post(API_URL, json=machine_id, timeout=2)
            if response.status_code == 200:
                data = response.json()
                if data["success"]:
                    live_values = data["data"]["LiveValues"]
                    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                    print(f"[{timestamp}] Nozzle: {live_values['nozzle_temperature']:.1f}°C, "
                          f"Front: {live_values['front_temperature']:.1f}°C, "
                          f"Pressure: {live_values['pressure']:.1f} bar")
                else:
                    print(f"Error: {data['error']}")
            else:
                print(f"HTTP Error: {response.status_code}")
        except Exception as e:
            print(f"Connection error: {e}")
        
        time.sleep(5)

if __name__ == "__main__":
    monitor_temperature()
```

#### Data Logging to CSV

```python
import requests
import csv
import time
from datetime import datetime

API_URL = "http://192.168.1.100:3001/api/v1/machine/event"

machine_id = {
    "machine_identification_unique": {
        "machine_identification": {
            "vendor": 1,
            "machine": 4
        },
        "serial": 4
    },
    "events": ["LiveValues", "State"]
}

def log_to_csv(filename="machine_data.csv", interval=10, duration=3600):
    """Log machine data to CSV file"""
    with open(filename, 'w', newline='') as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow([
            'Timestamp', 'Nozzle Temp (°C)', 'Front Temp (°C)', 
            'Pressure (bar)', 'Motor RPM', 'Power (W)'
        ])
        
        start_time = time.time()
        while time.time() - start_time < duration:
            try:
                response = requests.post(API_URL, json=machine_id, timeout=2)
                if response.status_code == 200:
                    data = response.json()
                    if data["success"]:
                        lv = data["data"]["LiveValues"]
                        writer.writerow([
                            datetime.now().isoformat(),
                            lv['nozzle_temperature'],
                            lv['front_temperature'],
                            lv['pressure'],
                            lv['motor_status']['rpm'],
                            lv['combined_power']
                        ])
                        csvfile.flush()
                        print(f"Logged data point at {datetime.now()}")
            except Exception as e:
                print(f"Error: {e}")
            
            time.sleep(interval)

if __name__ == "__main__":
    log_to_csv(filename="extruder_log.csv", interval=5, duration=3600)
    print("Logging complete!")
```

### JavaScript Examples

#### Basic Example - Get All Events (Node.js)

```javascript
const fetch = require('node-fetch');

const API_URL = 'http://192.168.1.100:3001/api/v1/machine/event';

const machineId = {
  machine_identification_unique: {
    machine_identification: {
      vendor: 1,  // QiTech
      machine: 4   // Extruder V1/V2
    },
    serial: 4
  },
  events: ["LiveValues", "State"]
};

async function getMachineData() {
  try {
    const response = await fetch(API_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(machineId)
    });

    const data = await response.json();
    
    if (data.success) {
      console.log('State:', JSON.stringify(data.data.State, null, 2));
      console.log('LiveValues:', JSON.stringify(data.data.LiveValues, null, 2));
    } else {
      console.error('Error:', data.error);
    }
  } catch (error) {
    console.error('Request failed:', error);
  }
}

getMachineData();
```

#### Get Specific Events (Node.js)

```javascript
const fetch = require('node-fetch');

const API_URL = 'http://192.168.1.100:3001/api/v1/machine/event';

async function getLiveValues() {
  const request = {
    machine_identification_unique: {
      machine_identification: {
        vendor: 1,
        machine: 2  // Winder V2
      },
      serial: 2
    },
    events: ["LiveValues"]
  };

  try {
    const response = await fetch(API_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request)
    });

    const data = await response.json();
    
    if (data.success) {
      const { puller_speed, spool_rpm, tension_arm_angle } = data.data.LiveValues;
      console.log(`Puller Speed: ${puller_speed} m/min`);
      console.log(`Spool RPM: ${spool_rpm} RPM`);
      console.log(`Tension Arm Angle: ${tension_arm_angle}°`);
    }
  } catch (error) {
    console.error('Error:', error);
  }
}

getLiveValues();
```

#### Continuous Monitoring (Node.js)

```javascript
const fetch = require('node-fetch');

const API_URL = 'http://192.168.1.100:3001/api/v1/machine/event';

const machineId = {
  machine_identification_unique: {
    machine_identification: {
      vendor: 1,
      machine: 4
    },
    serial: 4
  },
  events: ["LiveValues"]
};

async function monitorTemperature() {
  setInterval(async () => {
    try {
      const response = await fetch(API_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(machineId)
      });

      const data = await response.json();
      
      if (data.success) {
        const lv = data.data.LiveValues;
        const timestamp = new Date().toISOString();
        console.log(
          `[${timestamp}] Nozzle: ${lv.nozzle_temperature.toFixed(1)}°C, ` +
          `Front: ${lv.front_temperature.toFixed(1)}°C, ` +
          `Pressure: ${lv.pressure.toFixed(1)} bar`
        );
      } else {
        console.error('Error:', data.error);
      }
    } catch (error) {
      console.error('Connection error:', error.message);
    }
  }, 5000); // Every 5 seconds
}

monitorTemperature();
```

#### Browser Example (React)

```javascript
import React, { useState, useEffect } from 'react';

function MachineMonitor() {
  const [machineData, setMachineData] = useState(null);
  const [error, setError] = useState(null);

  const API_URL = 'http://192.168.1.100:3001/api/v1/machine/event';

  const machineId = {
    machine_identification_unique: {
      machine_identification: {
        vendor: 1,
        machine: 4
      },
      serial: 4
    },
    events: ["LiveValues", "State"]
  };

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch(API_URL, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(machineId)
        });

        const data = await response.json();
        
        if (data.success) {
          setMachineData(data.data);
          setError(null);
        } else {
          setError(data.error);
        }
      } catch (err) {
        setError(err.message);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 5000); // Update every 5 seconds

    return () => clearInterval(interval);
  }, []);

  if (error) {
    return <div>Error: {error}</div>;
  }

  if (!machineData) {
    return <div>Loading...</div>;
  }

  const { LiveValues, State } = machineData;

  return (
    <div>
      <h2>Machine Monitor</h2>
      <div>
        <h3>Mode: {State.mode_state.mode}</h3>
        <p>Nozzle Temperature: {LiveValues.nozzle_temperature.toFixed(1)}°C</p>
        <p>Front Temperature: {LiveValues.front_temperature.toFixed(1)}°C</p>
        <p>Pressure: {LiveValues.pressure.toFixed(1)} bar</p>
        <p>Power: {LiveValues.combined_power.toFixed(0)} W</p>
      </div>
    </div>
  );
}

export default MachineMonitor;
```

#### Data Logging to File (Node.js)

```javascript
const fetch = require('node-fetch');
const fs = require('fs');

const API_URL = 'http://192.168.1.100:3001/api/v1/machine/event';

const machineId = {
  machine_identification_unique: {
    machine_identification: {
      vendor: 1,
      machine: 4
    },
    serial: 4
  },
  events: ["LiveValues"]
};

async function logToFile(filename = 'machine_data.csv', intervalMs = 10000, durationMs = 3600000) {
  const stream = fs.createWriteStream(filename);
  
  // Write CSV header
  stream.write('Timestamp,Nozzle Temp (°C),Front Temp (°C),Pressure (bar),Motor RPM,Power (W)\n');

  const startTime = Date.now();
  
  const logInterval = setInterval(async () => {
    if (Date.now() - startTime > durationMs) {
      clearInterval(logInterval);
      stream.end();
      console.log('Logging complete!');
      return;
    }

    try {
      const response = await fetch(API_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(machineId)
      });

      const data = await response.json();
      
      if (data.success) {
        const lv = data.data.LiveValues;
        const row = [
          new Date().toISOString(),
          lv.nozzle_temperature,
          lv.front_temperature,
          lv.pressure,
          lv.motor_status.rpm,
          lv.combined_power
        ].join(',');
        
        stream.write(row + '\n');
        console.log(`Logged data point at ${new Date().toLocaleString()}`);
      }
    } catch (error) {
      console.error('Error:', error.message);
    }
  }, intervalMs);
}

// Log data every 5 seconds for 1 hour
logToFile('extruder_log.csv', 5000, 3600000);
```

## Best Practices

### Polling Frequency

- **High-frequency monitoring (1-5 seconds):** Suitable for critical parameters during production
- **Medium-frequency monitoring (10-30 seconds):** Suitable for general monitoring and dashboards
- **Low-frequency monitoring (1-5 minutes):** Suitable for data logging and trend analysis

Avoid polling faster than necessary to reduce network and server load.

### Request Optimization

**Request only needed event types:**
   ```json
   {
     "events": ["LiveValues"]
   }
   ```

### Example Production-Ready Python Logger

```python
import requests
import time
import logging
from datetime import datetime
from pathlib import Path

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)

class MachineDataLogger:
    def __init__(self, api_url, machine_id, interval=10):
        self.api_url = api_url
        self.machine_id = machine_id
        self.interval = interval
        self.session = requests.Session()
        self.consecutive_failures = 0
        self.max_consecutive_failures = 5
        
    def fetch_data(self):
        """Fetch data with retry logic"""
        request_body = {
            **self.machine_id,
            "events": ["LiveValues"]
        }
        
        try:
            response = self.session.post(
                self.api_url,
                json=request_body,
                timeout=5
            )
            response.raise_for_status()
            
            data = response.json()
            if not data["success"]:
                raise Exception(f"API Error: {data['error']}")
            
            self.consecutive_failures = 0
            return data["data"]["LiveValues"]
            
        except requests.exceptions.Timeout:
            self.consecutive_failures += 1
            logging.error("Request timed out")
            return None
        except requests.exceptions.ConnectionError:
            self.consecutive_failures += 1
            logging.error("Connection failed")
            return None
        except Exception as e:
            self.consecutive_failures += 1
            logging.error(f"Error: {e}")
            return None
    
    def run(self, duration_hours=None):
        """Run the logger"""
        start_time = time.time()
        
        while True:
            # Check if we should stop
            if duration_hours and (time.time() - start_time) > (duration_hours * 3600):
                logging.info("Logging duration reached, stopping")
                break
            
            # Check for too many consecutive failures
            if self.consecutive_failures >= self.max_consecutive_failures:
                logging.error("Too many consecutive failures, stopping")
                break
            
            # Fetch and log data
            data = self.fetch_data()
            if data:
                logging.info(f"Temperature: {data['nozzle_temperature']:.1f}°C, "
                           f"Pressure: {data['pressure']:.1f} bar")
                # Here you would save to database/file
            
            time.sleep(self.interval)

if __name__ == "__main__":
    logger = MachineDataLogger(
        api_url="http://192.168.1.100:3001/api/v1/machine/event",
        machine_id={
            "machine_identification_unique": {
                "machine_identification": {
                    "vendor": 1,
                    "machine": 1
                },
                "serial": 4
            }
        },
        interval=10
    )
    logger.run(duration_hours=24)
```

## Support

For issues, questions, or feature requests related to the Read-Only API:

- Check the [main documentation](./README.md)
- Review the [troubleshooting guide](./troubleshooting.md)
- Contact QiTech support