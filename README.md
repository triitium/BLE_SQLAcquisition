# BLE_SQLAcquisition

Proxy interface for the transfer of sensor data to a PostgreSQL database using Bluetooth Low Energy (BLE).

---

## Overview

BLE_SQLAcquisition is an asynchronous Rust application that acts as a bridge between a Bluetooth Low Energy (BLE) spectrometry device (for example, an ESP32) and a PostgreSQL database.

It continuously scans for a target device named `ESP32_ATH_SPEC`, connects to it, subscribes to its `NOTIFY` characteristic, and stores received spectrometry data directly into PostgreSQL.  
The service is designed to run in Docker and automatically handles reconnections and buffering of complete datasets before inserting them into the database.

---

## Features

- Automatic scanning for BLE devices with the expected name
- Automatic connection and reconnection if the device becomes unavailable
- Dynamic discovery of the BLE `NOTIFY` characteristic
- Data buffering until a full dataset of 3648 datapoints is received
- Direct PostgreSQL integration for inserting sensor data
- Designed to run as a container alongside a PostgreSQL service

---

## Requirements

- Docker and Docker Compose installed on the host system
- A BLE adapter accessible from the container
- An ESP32 device (or compatible) broadcasting under the name `ESP32_ATH_SPEC`

---

## Database Schema

The application expects the following table in the PostgreSQL database:

```sql
CREATE TABLE sensor_data (
    id SERIAL PRIMARY KEY,
    datapoints FLOAT8[]
);
```

---

## Configuration

The application reads its configuration from environment variables.  
When running through Docker Compose, these values are defined in the `docker-compose.yml`.

| Variable     | Required | Default        | Description                        |
|--------------|----------|----------------|------------------------------------|
| PG_USER      | yes      | —              | PostgreSQL username                |
| PG_PASSWORD  | yes      | —              | PostgreSQL password                |
| PG_HOST      | no       | database       | PostgreSQL host (container name in Compose) |
| PG_DBNAME    | no       | spectrometry   | Database name                      |
| PG_PORT      | no       | 5432           | PostgreSQL port                    |

---

## Running with Docker Compose

1. Build and start the services:

   ```bash
   docker compose up --build
   ```

2. The setup will start two containers:
   - `database` (PostgreSQL 15 instance)
   - `ble_proxy` (Rust BLE acquisition service)

3. The `ble_proxy` container depends on direct access to your BLE adapter.  
   Update the `devices` section in `docker-compose.yml` to point to the correct USB device path.  
   Example:

   ```yaml
   devices:
     - "/dev/bus/usb/001/005:/dev/bus/usb/001/005"
   ```

Use `lsusb` or `dmesg` on the host to find the correct device mapping.

---

## Architecture Diagram

+-------------+ BLE +----------------+ SQL +------------+
| | <----------------> | | ----------------> | |
| ESP32 | | ble_proxy | | PostgreSQL |
| (spectrometer) | container | | database |
| | | | | |
+-------------+ +----------------+ +------------+


Flow:
1. ESP32 broadcasts sensor data via BLE.
2. `ble_proxy` container scans, connects, and subscribes to the device.
3. Data is buffered until a full dataset is collected.
4. Full datasets are inserted into the PostgreSQL database.

---

## Usage Notes

- The service automatically retries if the ESP32 disconnects or is unavailable.  
- All collected spectrometry datasets (3648 datapoints each) are inserted into the `sensor_data` table.  
- To access the database from the host system:

  ```bash
  psql -h localhost -U external -d spectrometry
  ```

---

## Project Status

This is an early-stage implementation intended as a data acquisition proxy.  
Planned improvements include:

- Configurable device names and datapoint sizes
- Better error handling and retry backoff
- Support for multiple devices and concurrent acquisitions

