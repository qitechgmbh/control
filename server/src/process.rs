use std::{fs::File, io::Write, process};

pub fn init_process() {
    // write pid to file
    let pid = process::id();
    let mut pid_file =
        File::create("/tmp/qitech-control-server.pid").expect("Failed to create PID file");
    pid_file
        .write_all(pid.to_string().as_bytes())
        .expect("Failed to write PID to file");
}
