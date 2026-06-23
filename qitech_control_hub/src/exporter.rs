use std::{sync::Arc, time::Instant};
use chrono::{DateTime, Utc};
use tokio::{
    sync::broadcast,
    time::{Duration, timeout},
};
use clickhouse::{self, Client, Row};
use property::ExportedPropertySet;

pub struct Config {
    pub export_interval: Duration,
}

#[derive(Debug, Row, serde::Serialize, serde::Deserialize)]
struct PropertyRow<T> {
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    ts: DateTime<Utc>,
    ident: u64,
    name: String,
    value: T,
}

pub async fn run(
    client: Client,
    mut rx: broadcast::Receiver<Arc<ExportedPropertySet>>,
    config: Config,
) -> clickhouse::error::Result<()> {
    let mut last_export_ts = Instant::now();

    loop {
        let mut insert_float = client
            .insert::<PropertyRow<f64>>("properties_float")
            .await?;

        let mut insert_integer = client
            .insert::<PropertyRow<i64>>("properties_integer")
            .await?;

        let mut insert_bool = client
            .insert::<PropertyRow<bool>>("properties_bool")
            .await?;

        let mut insert_string = client
            .insert::<PropertyRow<heapless::String<128>>>("properties_string")
            .await?;

        loop {
            let now = Instant::now();

            if now.duration_since(last_export_ts) >= config.export_interval {
                println!("Exporting");

                tokio::try_join!(
                    insert_float.end(),
                    insert_integer.end(),
                    insert_bool.end(),
                    insert_string.end(),
                )?;
                
                last_export_ts = now;
                break;
            }

            if let Ok(result) = timeout(Duration::from_millis(100), rx.recv()).await {
                use broadcast::error::RecvError;
                let now = Utc::now();

                match result {
                    Ok(set) => {
                        for entry in &set.float {
                            insert_float.write(&PropertyRow {
                                ts: now,
                                ident: entry.ident,
                                name: entry.name.clone(),
                                value: entry.value,
                            }).await?;
                        }

                        for entry in &set.int {
                            insert_integer.write(&PropertyRow {
                                ts: now,
                                ident: entry.ident,
                                name: entry.name.clone(),
                                value: entry.value,
                            }).await?;
                        }

                        for entry in &set.bool {
                            insert_bool.write(&PropertyRow {
                                ts: now,
                                ident: entry.ident,
                                name: entry.name.clone(),
                                value: entry.value,
                            }).await?;
                        }

                        for entry in &set.string {
                            insert_string.write(&PropertyRow {
                                ts: now,
                                ident: entry.ident,
                                name: entry.name.clone(),
                                value: entry.value.clone(),
                            }).await?;
                        }
                    },
                    Err(e) => match e {
                        RecvError::Closed => return Ok(()),
                        RecvError::Lagged(count) => {
                            eprintln!("Lagged behind {count} messages!");
                            continue;
                        },
                    },
                }
            }
        }
    }
}
