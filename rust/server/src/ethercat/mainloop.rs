use crate::ethercat_drivers::cycle::pdu_once;
use anyhow::anyhow;
use ethercrab::std::ethercat_now;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

use crate::app_state::EthercatSetup;

pub fn pdu_loop(ethercat_setup: Arc<RwLock<Option<EthercatSetup>>>) -> Result<(), anyhow::Error> {
    let interval = Duration::from_nanos(1);

    // new tokio runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("spawn TX/RX runtime");

    loop {
        // drop the guard before the await
        {
            let ts_1 = ethercat_now();
            let mut guard = ethercat_setup.write();

            let setup = guard
                .as_mut()
                .ok_or_else(|| anyhow!("Ethercat setup not initialized"))?;

            pdu_once(setup, interval, &rt)?;
            let ts_2 = ethercat_now();
            log::trace!("PDU once took {} ns", ts_2 - ts_1);
        }
    }
}
