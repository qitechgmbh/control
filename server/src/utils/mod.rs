use std::process::Command;

pub fn start_dnsmasq() -> std::io::Result<()> {
    // action can be "start", "stop", or "restart"
    // let status = Command::new("systemctl")
    //     .arg("start")
    //     .arg("dnsmasq.service")
    //     .status()?;

    // if status.success() {
    //     tracing::info!("Successfully started dnsmasq");
    // } else {
    //     tracing::info!("Failed to start dnsmasq");
    // }
    Ok(())
}

pub fn stop_dnsmasq() -> std::io::Result<()> {
    // action can be "start", "stop", or "restart"
    // let status = Command::new("systemctl")
    //     .arg("stop")
    //     .arg("dnsmasq.service")
    //     .status()?;

    // if status.success() {
    //     tracing::info!("Successfully stopped");
    // } else {
    //     tracing::info!("Failed to stop dnsmasq");
    // }
    Ok(())
}
