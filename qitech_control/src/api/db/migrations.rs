//! Connects to ClickHouse and brings the `control_hub` tables up to date
//! (`migrations/*.sql`).

use super::DATABASE;

const URL: &str = "http://127.0.0.1:8123";

/// Connects to the local ClickHouse, creates or upgrades every table and returns a client
/// that uses the `control_hub` database.
pub async fn connect() -> clickhouse::Client {
    let client = clickhouse::Client::default()
        .with_url(URL)
        .with_user("default")
        .with_setting("allow_experimental_json_type", "1")
        .with_setting("input_format_binary_read_json_as_string", "1");

    execute(
        &client,
        &format!("CREATE DATABASE IF NOT EXISTS {DATABASE}"),
    )
    .await;

    execute(
        &client,
        include_str!("../../../migrations/init_machine_config_property_records.sql"),
    )
    .await;
    execute(
        &client,
        include_str!("../../../migrations/init_machine_state_property_records.sql"),
    )
    .await;
    execute(
        &client,
        include_str!("../../../migrations/init_machine_measurements.sql"),
    )
    .await;
    upgrade_machine_measurements_storage(&client).await;
    upgrade_machine_measurements_projections(&client).await;
    execute(
        &client,
        include_str!("../../../migrations/init_machine_events.sql"),
    )
    .await;
    execute(
        &client,
        include_str!("../../../migrations/init_machine_markers.sql"),
    )
    .await;

    client.with_database(DATABASE)
}

/// Upgrades a `machine_measurements` table from before the codecs (with the `by_time`
/// projection) in place. Skipped once the table has the new definition, so it does not
/// queue a mutation on every start.
async fn upgrade_machine_measurements_storage(client: &clickhouse::Client) {
    let create = table_definition(client, "machine_measurements").await;
    if !create.contains("by_time") && create.contains("DoubleDelta") {
        return;
    }

    tracing::info!("upgrading machine_measurements storage (drop projection, add codecs)");
    let upgrade = include_str!("../../../migrations/upgrade_machine_measurements_storage.sql");
    for statement in sql_statements(upgrade) {
        execute(client, &statement).await;
    }
}

/// Adds the history projections (see `init_machine_measurements.sql`) to a table created
/// before them, and fills them for parts written before they existed. The fill is a
/// background mutation; it is started again on the next start if it was interrupted.
async fn upgrade_machine_measurements_projections(client: &clickhouse::Client) {
    let create = table_definition(client, "machine_measurements").await;
    if !create.contains("p_5s") || !create.contains("p_1m") {
        tracing::info!("adding the history projections to machine_measurements");
        let upgrade =
            include_str!("../../../migrations/upgrade_machine_measurements_projections.sql");
        for statement in sql_statements(upgrade) {
            execute(client, &statement).await;
        }
    }

    let parts_missing: u64 = client
        .query(
            "SELECT count() FROM system.parts
             WHERE active AND database = ? AND table = 'machine_measurements'
               AND NOT (has(projections, 'p_5s') AND has(projections, 'p_1m'))",
        )
        .bind(DATABASE)
        .fetch_one()
        .await
        .unwrap();
    let fill_running: u64 = client
        .query(
            "SELECT count() FROM system.mutations
             WHERE database = ? AND table = 'machine_measurements'
               AND NOT is_done AND command ILIKE '%MATERIALIZE PROJECTION%'",
        )
        .bind(DATABASE)
        .fetch_one()
        .await
        .unwrap();
    if parts_missing > 0 && fill_running == 0 {
        tracing::info!(
            parts_missing,
            "filling the history projections in the background"
        );
        for projection in ["p_1m", "p_5s"] {
            execute(
                client,
                &format!(
                    "ALTER TABLE {DATABASE}.machine_measurements MATERIALIZE PROJECTION {projection}"
                ),
            )
            .await;
        }
    }
}

/// The `CREATE TABLE` statement ClickHouse currently holds for `table`.
async fn table_definition(client: &clickhouse::Client, table: &str) -> String {
    client
        .query("SELECT create_table_query FROM system.tables WHERE database = ? AND name = ?")
        .bind(DATABASE)
        .bind(table)
        .fetch_one()
        .await
        .unwrap()
}

async fn execute(client: &clickhouse::Client, sql: &str) {
    client.query(sql).execute().await.unwrap();
}

/// The statements of a migration file: `--` comment lines removed (they may contain
/// `;`), then split at `;`.
fn sql_statements(sql: &str) -> Vec<String> {
    let code = sql
        .lines()
        .filter(|line| !line.trim_start().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");
    code.split(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sql_statements;

    #[test]
    fn projection_migration_splits_into_its_alter_statements() {
        let sql = include_str!("../../../migrations/upgrade_machine_measurements_projections.sql");
        let statements = sql_statements(sql);
        assert_eq!(statements.len(), 2);
        assert!(
            statements
                .iter()
                .all(|s| s.contains("ADD PROJECTION IF NOT EXISTS"))
        );
    }

    #[test]
    fn upgrade_migration_splits_into_its_alter_statements() {
        let sql = include_str!("../../../migrations/upgrade_machine_measurements_storage.sql");
        let statements = sql_statements(sql);
        assert_eq!(statements.len(), 3);
        assert!(
            statements
                .iter()
                .all(|s| s.trim_start().starts_with("ALTER TABLE"))
        );
    }
}
