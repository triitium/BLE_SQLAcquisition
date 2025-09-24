# BLE_SQLAcquisition

Proxy interface for transferring spectrometry sensor data to a PostgreSQL database via Bluetooth Low Energy (BLE).

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
- An ESP32 (or compatible) broadcasting under the name `ESP32_ATH_SPEC`  

---

## Database Schema

The application expects the following table in PostgreSQL:

```sql
CREATE TABLE sensor_data (
    id SERIAL PRIMARY KEY,
    datapoints FLOAT8[]
);
```

---

## Configuration

All parameters are set via environment variables. When running through Docker Compose, these are defined in `docker-compose.yml`.

| Variable           | Required | Default        | Description |
|-------------------|----------|----------------|-------------|
| PG_USER            | yes      | —              | PostgreSQL username |
| PG_PASSWORD        | yes      | —              | PostgreSQL password |
| PG_HOST            | no       | database       | PostgreSQL host (container name in Compose) |
| PG_DBNAME          | no       | spectrometry   | Database name |
| PG_PORT            | no       | 5432           | PostgreSQL port |
| PG_TABLE           | no       | ESP32_athmos_spectro_001 | Table to insert data |
| BLE_DEVICE_NAME    | no       | ESP32_ATH_SPEC | Name of BLE device to connect |
| DATA_SIZE          | no       | 3648           | Number of datapoints per dataset |
| VALUE_BYTE_SIZE    | no       | 2              | Number of bytes per value (1, 2, or 4) |

---

## Running with Docker Compose

1. Build and start the services:

```bash
docker compose up --build
```

2. Two containers will start:
   - `database` (PostgreSQL 15 instance)
   - `ble_proxy` (Rust BLE acquisition service)

3. Ensure the `ble_proxy` container has access to your BLE adapter by updating the `devices` section in `docker-compose.yml`. Example:

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

