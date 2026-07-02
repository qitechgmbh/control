use clickhouse::{Client, query::Query};

use crate::api::{
    property::{Sample, Samples},
    types::{Aggregation, Ordering, Table, TimeSpan},
    utils,
};

pub struct Operation {
    pub table: Table,
    pub machine_uid: u64,
    pub property: String,
    pub time_span: TimeSpan,
    pub aggregation: Option<Aggregation>,
    pub ordering: Ordering,
    pub limit: u64,
}

impl Operation {
    pub async fn execute(self, client: &Client) -> Result<Samples, String> {
        let query = self.init_query(client);

        match self.table {
            Table::Float => {
                let samples = query
                    .fetch_all::<Sample<f64>>()
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(Samples::Float(samples))
            }

            Table::Integer => {
                let samples = query
                    .fetch_all::<Sample<i64>>()
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(Samples::Integer(samples))
            }
        }
    }

    pub fn init_query(&self, client: &Client) -> Query {
        let sql = self.init_sql();

        let mut query = client
            .query(&sql)
            .bind(self.machine_uid)
            .bind(&self.property);

        if let Some(from) = self.time_span.from {
            query = query.bind(utils::dt_to_ch_datetime64_ms(from));
        }

        if let Some(to) = self.time_span.to {
            query = query.bind(utils::dt_to_ch_datetime64_ms(to));
        }

        query = query.bind(self.limit);
        query
    }

    pub fn init_sql(&self) -> String {
        let mut sql = match &self.aggregation {
            Some(aggregation) => format!(
                r#"
                SELECT
                    toDateTime64(toStartOfInterval(ts, {}), 3) AS ts,
                    {} AS value
                FROM properties_{}
                WHERE ident = ?
                AND name = ?
                "#,
                aggregation.interval.to_ch(),
                aggregation.operation.to_ch(),
                self.table.to_str(),
            ),
            None => format!(
                r#"
                SELECT ts, value
                FROM properties_{}
                WHERE ident = ?
                AND name = ?
            "#,
                self.table.to_str()
            ),
        };

        // time filters
        if self.time_span.from.is_some() {
            sql.push_str(" AND ts >= toDateTime64(?, 3)");
        }

        if self.time_span.to.is_some() {
            sql.push_str(" AND ts <= toDateTime64(?, 3)");
        }

        // aggregation
        if self.aggregation.is_some() {
            sql.push_str("GROUP BY ts");
        }

        // ordering
        sql.push_str(" ORDER BY ts ");
        sql.push_str(self.ordering.to_ch());

        // limit
        sql.push_str(" LIMIT ?");

        sql
    }
}
