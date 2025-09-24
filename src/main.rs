use std::env;
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, Characteristic, CharPropFlags};
use btleplug::platform::Manager;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Load Postgres credentials from environment ---
    let user = env::var("PG_USER").expect("PG_USER not set");
    let password = env::var("PG_PASSWORD").expect("PG_PASSWORD not set");
    let host = env::var("PG_HOST").unwrap_or_else(|_| "database".into());
    let dbname = env::var("PG_DBNAME").unwrap_or_else(|_| "spectrometry".into());
    let port = env::var("PG_PORT").unwrap_or_else(|_| "5432".into());

    let conn_str = format!(
        "host={} user={} password={} dbname={} port={}",
        host, user, password, dbname, port
    );

    // --- Connect to PostgreSQL ---
    let (pg_client, connection) = tokio_postgres::connect(&conn_str, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Postgres connection error: {}", e);
        }
    });

    // --- Setup BLE ---
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).expect("No BLE adapter found");

    central.start_scan(ScanFilter::default()).await?;
    sleep(Duration::from_secs(2)).await;

    // Connect to ESP32 device
    let peripherals = central.peripherals().await?;
    let peripheral = peripherals.into_iter()
        .find(|p| p.properties().local_name.iter().flatten().any(|n| n.contains("ESP32_ATH_SPEC")))
        .expect("BLE device not found");

    peripheral.connect().await?;
    println!("Connected to ESP32_ATH_SPEC!");

    // Characteristic UUID for data notifications
    let char_uuid = Uuid::parse_str("abcdef01-1234-5678-1234-56789abcdef0")?;
    let characteristic = Characteristic {
        uuid: char_uuid,
        properties: CharPropFlags::NOTIFY,
        descriptors: vec![],
    };

    let mut buffer: Vec<u8> = Vec::new();
    let total_values = 3648;

    peripheral.subscribe(&characteristic).await?;
    peripheral.on_notification(Box::new(move |notif| {
        buffer.extend_from_slice(&notif.value);

        if buffer.len() >= total_values * 2 {
            // Convert u16 -> f32 for PostgreSQL REAL[] column
            let values: Vec<f32> = buffer.chunks(2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]) as f32)
                .collect();

            let client = pg_client.clone();
            tokio::spawn(async move {
                if let Err(e) = client.execute(
                    "INSERT INTO sensor_data (datapoints) VALUES ($1)",
                    &[&values]
                ).await {
                    eprintln!("Failed to insert into PostgreSQL: {}", e);
                } else {
                    println!("Inserted {} values into PostgreSQL", values.len());
                }
            });

            buffer.clear();
        }
    })).await;

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
