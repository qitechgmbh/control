# Logging

The backend logs through the [tracing](https://docs.rs/tracing) framework. Across the code, events are recorded at the usual levels — `error`, `warn`, `info`, `debug`, and `trace` — using tracing's macros.

## Where the logs go

**In development**, log events appear in the console of the backend you started (a `tracing-fmt` feature, on by default, formats them for the terminal).

**On a device**, the backend runs as the systemd service `qitech-control-server`, whose standard output and error are sent to the **system journal**. View them with:

```bash
# recent logs for the service
journalctl -u qitech-control-server

# follow live
journalctl -u qitech-control-server -f
```

## Exporting logs for support

The operator interface can export the recent system-journal logs to a file (via `journalctl`), which is the quickest way to capture what happened for a bug report.
