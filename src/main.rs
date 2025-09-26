use std::env;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, CharPropFlags};
use btleplug::platform::Manager;
use tokio::time::{sleep, Duration};
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user = env::var("PG_USER").expect("PG_USER not set");
    let password = env::var("PG_PASSWORD").expect("PG_PASSWORD not set");
    let host = env::var("PG_HOST").unwrap_or_else(|_| "database".into());
    let dbname = env::var("PG_DBNAME").unwrap_or_else(|_| "spectrometry".into());
    let port = env::var("PG_PORT").unwrap_or_else(|_| "5432".into());
    let table_name = env::var("PG_TABLE")
        .unwrap_or_else(|_| "ESP32_athmos_spectro_001".into());

    let device_name = env::var("BLE_DEVICE_NAME").unwrap_or_else(|_| "ESP32_ATH_SPEC".into());
    let package_size: usize = env::var("DATA_SIZE")
        .unwrap_or_else(|_| "3648".into())
        .parse()
        .expect("DATA_SIZE must be a number");

    let value_byte_size: usize = env::var("VALUE_BYTE_SIZE")
        .unwrap_or_else(|_| "2".into())
        .parse()
        .expect("VALUE_BYTE_SIZE must be a number");

    let conn_str = format!(
        "host={} user={} password={} dbname={} port={}",
        host, user, password, dbname, port
    );

    let (pg_client, connection) = tokio_postgres::connect(&conn_str, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {}", e);
        }
    });

    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).expect("No BLE adapter found");

    loop {
        let peripheral = loop {
            println!("Scanning for {}...", device_name);
            central.start_scan(ScanFilter::default()).await?;
            sleep(Duration::from_secs(5)).await;

            let peripherals = central.peripherals().await?;
            if let Some(p) = peripherals.into_iter().find(|p| {
                p.properties()
                    .local_name
                    .iter()
                    .flatten()
                    .any(|n| n.contains(&device_name))
            }) {
                println!("Found {}!", device_name);
                break p;
            } else {
                println!("Device not found, retrying in 30s...");
                sleep(Duration::from_secs(30)).await;
            }
        };

        if let Err(e) = peripheral.connect().await {
            eprintln!("Failed to connect: {}", e);
            sleep(Duration::from_secs(30)).await;
            continue;
        }
        println!("Connected to {}!", device_name);

        let characteristics = peripheral.characteristics();
        let indicate_char = match characteristics.into_iter()
            .find(|c| c.properties.contains(CharPropFlags::INDICATE)) 
        {
            Some(c) => c,
            None => {
                eprintln!("No INDICATE characteristic found on device");
                peripheral.disconnect().await.ok();
                sleep(Duration::from_secs(30)).await;
                continue;
            }
        };

        println!("Subscribing to characteristic with UUID: {}", indicate_char.uuid);

        let mut buffer: Vec<u8> = Vec::new();
        if let Err(e) = peripheral.subscribe(&indicate_char).await {
            eprintln!("Failed to subscribe: {}", e);
            peripheral.disconnect().await.ok();
            sleep(Duration::from_secs(30)).await;
            continue;
        }

        let client = pg_client.clone();
        let table_name = table_name.clone();
        let value_byte_size_clone = value_byte_size;
        peripheral.on_notification(Box::new(move |notif| {
            buffer.extend_from_slice(&notif.value);

            if buffer.len() >= package_size * value_byte_size_clone {
                let values: Vec<f32> = buffer.chunks(value_byte_size_clone)
                    .map(|b| match value_byte_size_clone {
                        1 => b[0] as f32,
                        2 => u16::from_le_bytes([b[0], b[1]]) as f32,
                        4 => f32::from_le_bytes([b[0], b[1], b[2], b[3]]),
                        _ => panic!("Unsupported VALUE_BYTE_SIZE"),
                    })
                    .collect();

                let client = client.clone();
                let table_name = table_name.clone();
                tokio::spawn(async move {
                    let query = format!("INSERT INTO {} (datapoints) VALUES ($1)", table_name);
                    if let Err(e) = client.execute(&query, &[&values]).await {
                        eprintln!("Failed to insert into PostgreSQL: {}", e);
                    } else {
                        println!("Inserted {} values into PostgreSQL", values.len());
                    }
                });

                buffer.clear();
            }
        })).await;

        loop {
            if !peripheral.is_connected().await? {
                println!("Device disconnected, will rescan...");
                break;
            }
            sleep(Duration::from_secs(5)).await;
        }
    }
}
