use std::{cell::RefCell, rc::Rc};

use qitech_lib::{ethercat_hal::{EtherCATThreadChannel, devices::{EthercatDevice, downcast_rc_refcell}, machine_ident_read::MachineDeviceInfo}, modbus::ModbusDevice};

#[derive(Clone)]
pub struct IdentifiedEthercat {
    pub hw: Rc<RefCell<dyn EthercatDevice>>,
    pub ident: MachineDeviceInfo,
}

#[derive(Clone)]
pub struct IdentifiedModbus {
    pub hw: Rc<RefCell<dyn ModbusDevice>>,
}

#[derive(Clone)]
pub enum Hardware {
    Ethercat(IdentifiedEthercat),
    Modbus(IdentifiedModbus),
}

#[derive(Clone)]
pub struct MachineHardware {
    pub hw: Vec<Hardware>,
    pub ethercat_interface: Option<EtherCATThreadChannel>,
}

impl MachineHardware {
    pub fn try_get_ethercat_device_by_index<T>(
        &self,
        index: usize,
    ) -> Result<Rc<RefCell<T>>, anyhow::Error>
    where
        T: EthercatDevice,
    {
        let hw = self.hw.get(index);
        let hw = match hw {
            Some(hw) => hw,
            None => return Err(anyhow::anyhow!("index {} not found in hardware", index)),
        };

        let identified_ethercat = match hw {
            Hardware::Ethercat(rc_ecat) => rc_ecat,
            _ => {
                return Err(anyhow::anyhow!(
                    "index {} not an ethercat device in hardware",
                    index
                ));
            }
        };
        
        downcast_rc_refcell::<T>(identified_ethercat.hw.clone())
    }

    pub fn try_get_ethercat_meta_by_role(&self, role: u16) -> Result<u16, anyhow::Error> {
        for i in 0..self.hw.len() {
            let hardware = self.hw.get(i).expect("try_get_ethercat_device_by_role failed to get hardware even though i is in range of len??????");
            match hardware {
                Hardware::Ethercat(identified_ethercat) => {
                    if identified_ethercat.ident.role == role {
                        return Ok(identified_ethercat.ident.device_address);
                    }
                    continue;
                }
                _ => continue,
            }
        }
        Err(anyhow::anyhow!(
            "index {} not an ethercat device in hardware",
            role
        ))
    }

    pub fn downcast_serial_rc_refcell<T: 'static>(
        dev: Rc<RefCell<dyn ModbusDevice>>,
    ) -> Result<Rc<RefCell<T>>, anyhow::Error> {
        // Check if the inner type is actually T
        let is_t = dev.borrow().as_any().is::<T>();
        if !is_t {
            return Err(anyhow::anyhow!("Type mismatch in hardware downcast"));
        }
        // Since we verified the type above, we can use raw pointers.
        let raw_trait_ptr = Rc::into_raw(dev);
        // We cast the fat pointer to a thin pointer of the concrete RefCell<T>
        let raw_concrete_ptr = raw_trait_ptr as *const RefCell<T>;
        unsafe { Ok(Rc::from_raw(raw_concrete_ptr)) }
    }

    pub fn try_get_serial_device_by_index<T: 'static>(
        &self,
        index: usize,
    ) -> Result<Rc<RefCell<T>>, anyhow::Error> {
        let hw = self.hw.get(index).unwrap().clone();
        match hw {
            Hardware::Modbus(identified_modbus) => {
                Self::downcast_serial_rc_refcell::<T>(identified_modbus.hw)
            }
            _ => Err(anyhow::anyhow!(
                "index {} not an modbus device in hardware",
                index
            )),
        }
    }

    pub fn try_get_ethercat_device_and_addr_by_role<T>(
        &self,
        role: u16,
    ) -> Result<(Rc<RefCell<T>>, u16), anyhow::Error>
    where
        T: EthercatDevice,
    {
        for i in 0..self.hw.len() {
            let hardware = self.hw.get(i).expect("try_get_ethercat_device_by_role failed to get hardware even though i is in range of len??????");
            match hardware {
                Hardware::Ethercat(identified_ethercat) => {
                    if identified_ethercat.ident.role == role {
                        let res = downcast_rc_refcell::<T>(identified_ethercat.hw.clone())?;
                        return Ok((res, identified_ethercat.ident.device_address));
                    }
                    continue;
                }
                _ => continue,
            }
        }
        Err(anyhow::anyhow!(
            "index {} not an ethercat device in hardware",
            role
        ))
    }

    pub fn try_get_ethercat_device_by_role<T>(
        &self,
        role: u16,
    ) -> Result<Rc<RefCell<T>>, anyhow::Error>
    where
        T: EthercatDevice,
    {
        for i in 0..self.hw.len() {
            let hardware = self.hw.get(i).expect("try_get_ethercat_device_by_role failed to get hardware even though i is in range of len??????");
            match hardware {
                Hardware::Ethercat(identified_ethercat) => {
                    if identified_ethercat.ident.role == role {
                        return downcast_rc_refcell::<T>(identified_ethercat.hw.clone());
                    }
                    continue;
                }
                _ => continue,
            }
        }
        Err(anyhow::anyhow!(
            "index {} not an ethercat device in hardware",
            role
        ))
    }
}
