# Requests/Responses

## 0x0000 - Serial Number

### Read

| Field    | Type |
|----------|------|
| request  | void |
| response | u32  |


## 0x0001 - Device Identification

### Read

| Field    | Type |
|----------|------|
| request  | void |
| response | u8   |

### Write

| Field    | Type |
|----------|------|
| request  | u8   |
| response | u4   |


## 0x0007 - Hardware Version

### Read

| Field    | Type |
|----------|------|
| request  | void |
| response | u8   |


## 0x0008 - Software Version

### Read

| Field    | Type |
|----------|------|
| request  | void |
| response | u128 |


## 0x0009 - Sealing Switch Status

### Read

| Field    | Type |
|----------|------|
| request  | void |
| response | bool |