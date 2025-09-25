# BLE_SQLAcquisition

Proxy interface for transferring sensor data to a PostgreSQL database via Bluetooth Low Energy (BLE).

---

## Overview

**BLE_SQLAcquisition** is an asynchronous Rust application that bridges a Bluetooth Low Energy (BLE) spectrometry device (e.g., ESP32) with a PostgreSQL database.

It continuously scans for a specified target device, connects to it, subscribes to its `INDICATE` characteristic, buffers incoming sensor data, and inserts full datasets into PostgreSQL.  
The service is designed to run in Docker and automatically handles reconnections and buffering.

---

## Features

- Automatic scanning for BLE devices with a configurable name  
- Automatic connection and reconnection if the device becomes unavailable  
- Dynamic discovery of the BLE `INDICATE` characteristic  
- Data buffering until a full dataset of configurable size is received  
- Direct PostgreSQL integration for inserting sensor data  
- Configurable via environment variables  
- Designed to run as a container alongside a PostgreSQL service  

---

## Requirements

- Docker and Docker Compose installed on the host system  
- A BLE adapter accessible from the container  
- An ESP32 (or compatible) broadcasting under the name specified in `docker-compose.yaml`  

---

## Database Schema

The application expects the following table in PostgreSQL:

```sql
CREATE TABLE PG_TABLE (
    id SERIAL PRIMARY KEY,
    datapoints REAL[],
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

---

## Configuration

All parameters are set via environment variables. When running through Docker Compose, these are defined in `docker-compose.yaml`.

| Variable           | Description |
|-------------------|-------------|
| PG_USER            | PostgreSQL username |
| PG_PASSWORD        | PostgreSQL password |
| PG_HOST            | PostgreSQL host (container name in Compose) |
| PG_DBNAME          | Database name |
| PG_PORT            | PostgreSQL port |
| PG_TABLE           | Table to insert data |
| BLE_DEVICE_NAME    | Name of BLE device to connect |
| DATA_SIZE          | Number of datapoints per dataset |
| VALUE_BYTE_SIZE    | Number of bytes per value (1, 2, or 4) |

---

## Running with Docker Compose

1. Build and start the services:

```bash
docker compose up --build
```

2. The container will start:
   - `ble_proxy` (Rust BLE acquisition service)

3. Ensure the `ble_proxy` container has access to your BLE adapter by updating the `devices` section in `docker-compose.yaml`. Example:

```yaml
devices:
  - "/dev/bus/usb/001/005:/dev/bus/usb/001/005"
```

Use `lsusb` or `dmesg` on the host to find the correct device path.

---

## Usage Notes

- The service retries automatically if the ESP32 disconnects or is unavailable
- All collected datasets (e.g., 3648 datapoints) are inserted into the `PG_TABLE`
- Values are parsed according to `VALUE_BYTE_SIZE` (1, 2, or 4 bytes)

---

## Project Status

Early-stage implementation intended as a BLE-to-SQL proxy.  
Planned improvements:

- Configurable characteristic types
- Better error handling with exponential backoff
- Support for multiple devices and concurrent acquisitions
- Support for different database systems

---

