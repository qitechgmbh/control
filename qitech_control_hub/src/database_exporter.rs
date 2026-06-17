use std::{sync::Arc, time::Instant};
use chrono::{DateTime, Utc};
use tokio::{
    sync::broadcast,
    time::{Duration, timeout},
};
use clickhouse::{Client, Row};
use property::ExportedPropertySet;

#[derive(Debug, Row, serde::Serialize, serde::Deserialize)]
struct PropertyRow<T> {
    ts: DateTime<Utc>,
    ident: u64,
    name: String,
    value: T,
}

pub async fn run(
    client: Client,
    mut rx: broadcast::Receiver<Arc<ExportedPropertySet>>,
) {
    let mut last_export_ts = Instant::now();
    let export_interval = Duration::from_secs_f64(2.0);

    loop {
        let mut insert_float = client
            .insert::<PropertyRow<f64>>("properties_float")
            .await.expect("LMAO");

        let mut insert_integer = client
            .insert::<PropertyRow<i64>>("properties_integer")
            .await.expect("LMAO");

        let mut insert_bool = client
            .insert::<PropertyRow<bool>>("properties_bool")
            .await.expect("LMAO");

        let mut insert_string = client
            .insert::<PropertyRow<heapless::String<128>>>("properties_string")
            .await.expect("LMAO");

        loop {
            let now = Instant::now();

            if last_export_ts.duration_since(now) >= export_interval {
                insert_float.end().await.expect("___");
                insert_integer.end().await.expect("___");
                insert_bool.end().await.expect("___");
                insert_string.end().await.expect("___");
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
                            }).await.expect("REMOVE LATER");
                        }

                        for entry in &set.int {
                            insert_integer.write(&PropertyRow {
                                ts: now,
                                ident: entry.ident,
                                name: entry.name.clone(),
                                value: entry.value,
                            }).await.expect("REMOVE LATER");
                        }

                        for entry in &set.bool {
                            insert_bool.write(&PropertyRow {
                                ts: now,
                                ident: entry.ident,
                                name: entry.name.clone(),
                                value: entry.value,
                            }).await.expect("REMOVE LATER");
                        }

                        for entry in &set.string {
                            insert_string.write(&PropertyRow {
                                ts: now,
                                ident: entry.ident,
                                name: entry.name.clone(),
                                value: entry.value.clone(),
                            }).await.expect("REMOVE LATER");
                        }
                    },
                    Err(e) => match e {
                        RecvError::Closed => return,
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
