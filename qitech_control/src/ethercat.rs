use std::{cell::RefCell, rc::Rc};

use anyhow::{Context, Result, anyhow, bail};
use bitvec::{order::Lsb0, slice::BitSlice};
use qitech_lib::ethercat_hal::{
    EtherCATControl, EtherCATState, EtherCATThreadChannel, MetaSubdevice,
    controller::{TripleBufConsumer, TripleBufProducer},
    devices::{EthercatDevice, device_from_subdevice_identity_rc},
    init_ethercat,
    machine_ident_read::MachineDeviceInfo,
};

// TODO: this type prob want to live in qitech_lib
pub type EtherCATDevice = Rc<RefCell<dyn EthercatDevice>>;

#[derive(Default)]
pub struct EtherCAT(Option<EtherCATControl<TripleBufConsumer, TripleBufProducer>>);

impl EtherCAT {
    // TODO: count this be part of qitech_lib

    pub fn write_inputs(&mut self, subdevices: &mut Vec<EtherCATDevice>) -> Result<()> {
        let ecat = self.check_online()?;

        assert!(ecat.controller.subdevice_count == subdevices.len());
        let inputs = ecat.app_handle.get_inputs();

        for i in 0..ecat.controller.subdevice_count {
            let meta_dev = ecat.controller.subdevices[i];
            let subdevice = &mut subdevices[i];
            let input_slice = &inputs[meta_dev.start_tx..meta_dev.end_tx];
            let input_bits_slice = BitSlice::<u8, Lsb0>::from_slice(input_slice);

            {
                let mut subdevice = subdevice.borrow_mut();
                let _res = subdevice.input(input_bits_slice);
                let _res = subdevice.input_post_process();
            }
        }

        Ok(())
    }

    pub fn write_outputs(&mut self, subdevices: &mut Vec<EtherCATDevice>) -> Result<()> {
        let ecat = self.check_online()?;

        assert!(ecat.controller.subdevice_count == subdevices.len());

        let outputs = ecat.app_handle.write_outputs();

        for i in 0..ecat.controller.subdevice_count {
            let meta_dev = ecat.controller.subdevices[i];
            let subdevice = &mut subdevices[i];
            let output_slice = &mut outputs[meta_dev.start_rx..meta_dev.end_rx];
            let output_bits = BitSlice::<u8, Lsb0>::from_slice_mut(output_slice);

            {
                let mut subdevice = subdevice.borrow_mut();
                let _res = subdevice.output(output_bits);
                let _res = subdevice.output_pre_process();
            }
        }

        ecat.app_handle.send_outputs();

        Ok(())
    }

    pub fn goto_state(&mut self, state: EtherCATState) -> Result<()> {
        let ecat = self.check_online()?;

        if let Err(e) = ecat.channel.request_state_change(state) {
            return self.fail(anyhow!("State change failed: {e}"));
        }

        Ok(())
    }

    pub fn create_devices(&mut self) -> Result<Vec<(MetaSubdevice, EtherCATDevice)>> {
        let ecat = self.check_online()?;
        let mut devices = vec![];

        for i in 0..ecat.controller.subdevice_count {
            let meta = ecat.controller.subdevices[i];
            let dev = device_from_subdevice_identity_rc(meta)
                .context("Failed to create EtherCAT device from meta")?;
            devices.push((meta, dev));
        }

        Ok(devices)
    }

    pub fn read_device_identification_from_eeprom(&mut self) -> Result<Vec<MachineDeviceInfo>> {
        let ecat = self.check_online()?;
        ecat.channel.read_device_identification_from_eeprom()
    }

    pub fn get_channel(&mut self) -> Option<EtherCATThreadChannel> {
        let ecat = self.check_online().ok()?;
        Some(ecat.channel.clone())
    }

    pub fn init(&mut self, interface: &str) -> Result<()> {
        if self.is_online() {
            bail!("EtherCAT already online!");
        }

        self.0 = Some(init_ethercat(interface)); // TODO: init_ethercat should return a Result<>
        self.goto_state(EtherCATState::PreOp)?;

        let _ = self.check_online()?;
        Ok(())
    }

    pub fn is_online(&self) -> bool {
        self.0.is_some()
    }

    fn fail(&mut self, e: anyhow::Error) -> Result<()> {
        tracing::error!("EtherCAT Error: {e}");
        self.0 = None;
        Err(e)
    }

    fn check_online(
        &mut self,
    ) -> Result<&mut EtherCATControl<TripleBufConsumer, TripleBufProducer>> {
        self.0.as_mut().ok_or(anyhow!("EtherCAT offline!"))
    }
}
