use std::time::Instant;
use chrono::{DateTime, Utc};
use property::{PropertySetView, StringPropertyValue};
use tokio::{
    sync::broadcast,
    time::{Duration, timeout},
};

use clickhouse::{self, Client, Row, insert::Insert};

use crate::{PropertyMessage, SharedState};

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

struct Inserts {
    pub float: Insert<PropertyRow<f64>>,
    pub integer: Insert<PropertyRow<i64>>,
    pub boolean: Insert<PropertyRow<bool>>,
    pub string: Insert<PropertyRow<StringPropertyValue>>,
}

impl Inserts {
    pub async fn new(client: &Client) -> clickhouse::error::Result<Self> {
        let float = client
            .insert::<PropertyRow<f64>>("properties_float")
            .await?;

        let integer = client
            .insert::<PropertyRow<i64>>("properties_integer")
            .await?;

        let boolean = client
            .insert::<PropertyRow<bool>>("properties_bool")
            .await?;

        let string = client
            .insert::<PropertyRow<StringPropertyValue>>("properties_string")
            .await?;
        
        Ok(Self { float, integer, boolean, string })
    }

    pub async fn end(self) -> clickhouse::error::Result<()> {
        tokio::try_join!(
            self.float.end(),
            self.integer.end(),
            self.boolean.end(),
            self.string.end(),
        )?;

        Ok(())
    }
}

pub async fn run(
    state: SharedState,
    mut rx: broadcast::Receiver<PropertyMessage>,
    config: Config,
) -> clickhouse::error::Result<()> {
    let mut last_export_ts = Instant::now();

    loop {
        let mut inserts = Inserts::new(&state.client).await?;

        loop {
            let now = Instant::now();

            if now.duration_since(last_export_ts) >= config.export_interval {
                println!("Exporting");
                inserts.end().await?;
                last_export_ts = now;
                break;
            }

            if let Ok(result) = timeout(Duration::from_millis(100), rx.recv()).await {
                use broadcast::error::RecvError;
                let now = Utc::now();

                match result {
                    Ok(set) => map_message(&mut inserts, set, now).await?,
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

async fn map_message(
    inserts: &mut Inserts, 
    msg: PropertyMessage,
    now: DateTime<Utc>,
) -> clickhouse::error::Result<()> {
    // use PropertyMessage::*;

    let view = match &msg {
        PropertyMessage::Native(set) => PropertySetView::native_dirty(&set),
        PropertyMessage::Exported(set) => PropertySetView::exported(&set),
    };

    for entry in view.float {
        inserts.float.write(&PropertyRow {
            ts: now,
            ident: entry.ident,
            name: entry.name.into(),
            value: *entry.value,
        }).await?;
    }

    for entry in view.integer {
        inserts.integer.write(&PropertyRow {
            ts: now,
            ident: entry.ident,
            name: entry.name.into(),
            value: *entry.value,
        }).await?;
    }

    for entry in view.boolean {
        inserts.boolean.write(&PropertyRow {
            ts: now,
            ident: entry.ident,
            name: entry.name.into(),
            value: *entry.value,
        }).await?;
    }

    for entry in view.string {
        inserts.string.write(&PropertyRow {
            ts: now,
            ident: entry.ident,
            name: entry.name.into(),
            value: entry.value.clone(),
        }).await?;
    }

    Ok(())
}
