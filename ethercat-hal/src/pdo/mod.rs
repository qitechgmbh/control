pub mod analog_input;
pub mod basic;
pub mod el252x;
pub mod el32xx;
pub mod el40xx;
pub mod el5152;
pub mod el70x1;
use crate::coe::Configuration;
use bitvec::prelude::*;

/// This trait allows to know the size of a PDO object in bits.
///
/// Can be derived with the [`ethercat_hal_derive::PdoObject`] macro.
///
/// Example:
/// ```ignore
/// #[derive(PdoObject)]
/// #[pdo_object(bits = 1)]
/// struct BoolPdoObject {
///    pub value: bool,
/// }
/// ````
///
/// Equivalent:
/// ```ignore
/// struct BoolPdoObject {
///   pub value: bool,
/// }
///
/// impl PdoObject for BoolPdoObject {
///   fn size(&self) -> usize {
///     1
///    }
/// }
/// ```
pub trait PdoObject {
    /// size in bits
    fn size(&self) -> usize;
}

/// This trait adds the [`TxPdoObject::read`] method which is used to decode the PDO bit array
///
/// Example:
/// ```ignore
/// impl TxPdoObject for BoolPdoObject {
///     fn read(&mut self, buffer: &BitSlice<u8, Lsb0>) {
///         self.value = buffer[0].into();
///     }
/// }
/// ````
pub trait TxPdoObject: PdoObject {
    fn read(&mut self, buffer: &BitSlice<u8, Lsb0>);
}

/// This trait adds the [`RxPdoObject::write`] method which is used to encode the PDO bit array
///
/// Example:
/// ```ignore
/// impl RxPdoObject for BoolPdoObject {
///     fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
///         buffer.set(0, self.value);
///     }
/// }
/// ```
pub trait RxPdoObject: PdoObject {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>);
}

/// This trait should be implemented on enums that represet an specific PDO assignment.
///
/// Example:
/// ```ignore
/// #[derive(Debug, Clone)]
/// pub enum EL3001PredefinedPdoAssignment {
///     Standard,
///     Compact,
/// }
///
/// impl PredefinedPdoAssignment<EL3001TxPdo, EL3001RxPdo> for EL3001PredefinedPdoAssignment {
///     fn txpdo_assignment(&self) -> EL3001TxPdo {
///         match self {
///             EL3001PredefinedPdoAssignment::Standard => EL3001TxPdo {
///                 ai_standard: Some(AiStandard::default()),
///                 ai_compact: None,
///             },
///             EL3001PredefinedPdoAssignment::Compact => EL3001TxPdo {
///                 ai_standard: None,
///                 ai_compact: Some(AiCompact::default()),
///             },
///         }
///     }
///
///     fn rxpdo_assignment(&self) -> EL3001RxPdo {///
///        match self {
///            EL3001PredefinedPdoAssignment::Standard => EL3001RxPdo {},
///            EL3001PredefinedPdoAssignment::Compact => EL3001RxPdo {},
///        }
///    }
/// }
/// ```
pub trait PredefinedPdoAssignment<TXPDOA, RXPDOA> {
    fn txpdo_assignment(&self) -> TXPDOA;
    fn rxpdo_assignment(&self) -> RXPDOA;
}

/// This trait is used on struct that hold all the possible PDO objects
///
/// Since the different PDO object can be enabled/disable with the PDO assignments they are wrapped in an `Option`.
///
/// Can be derived using the [`ethercat_hal_derive::RxPdo`] macro with the `#[pdo_object_index]` attribute.
///
/// ```ignore
/// #[derive(Debug, Clone, RxPdo)]
/// pub struct EL2002RxPdo {
///     #[pdo_object_index(0x1600)]
///     pub channel1: Option<BoolPdoObject>,
///     #[pdo_object_index(0x1601)]
///     pub channel2: Option<BoolPdoObject>,
/// }
/// ```
pub trait RxPdo: Configuration {
    /// Get objects return an array of optinal references to the PDO objects
    ///
    /// This method is commonly derived using the [`ethercat_hal_derive::RxPdo`] macro.
    fn get_objects(&self) -> Box<[Option<&dyn crate::pdo::RxPdoObject>]>;

    /// Calculating the size of the PDO assignment in bits
    ///
    /// Only the PDO objects that are Some(_) are counted.
    ///
    /// Will always be rouneded to the next byte since not all bits are always used but space in the PDU is reserved on byte basis.
    fn size(&self) -> usize {
        let used_bits = self
            .get_objects()
            .iter()
            .map(|object| match object {
                Some(object) => object.size(),
                None => 0,
            })
            .sum::<usize>();
        let padding = match used_bits % 8 {
            0 => 0,
            _ => 8 - used_bits % 8,
        };
        used_bits + padding
    }

    /// Will give the mutable PDU bit array to the PDO objects to encode the data
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) -> Result<(), anyhow::Error> {
        let mut bit_offset = 0;
        for object in self.get_objects() {
            if let Some(object) = object {
                let end_bit_index = bit_offset + object.size();

                // check if end_bit_index is out of bounds
                if end_bit_index > buffer.len() {
                    return Err(anyhow::anyhow!(
                        "[{}::RxPdo::write] Range {}..{} ({}bits) is out of bounds for buffer with length {}",
                        module_path!(),
                        bit_offset,
                        end_bit_index,
                        object.size(),
                        buffer.len()
                    ));
                }

                object.write(&mut buffer[bit_offset..end_bit_index]);
                bit_offset += object.size();
            }
        }
        Ok(())
    }
}

/// This trait is used on struct that hold all the possible PDO objects
///
/// Since the different PDO object can be enabled/disable with the PDO assignments they are wrapped in an `Option`.
///
/// Can be derived using the [`ethercat_hal_derive::RxPdo`] macro with the `#[pdo_object_index]` attribute.
///
/// ```ignore
/// #[derive(Debug, Clone, RxPdo)]
/// pub struct EL1002TxPdo {
///     #[pdo_object_index(0x1A00)]
///     pub channel1: Option<BoolPdoObject>,
///     #[pdo_object_index(0x1A01)]
///     pub channel2: Option<BoolPdoObject>,
/// }
/// ```
pub trait TxPdo: Configuration {
    /// Get objects return an array of optinal references to the PDO objects
    ///
    /// This method is commonly derived using the [`ethercat_hal_derive::TxPdo`] macro.
    fn get_objects(&self) -> Box<[Option<&dyn TxPdoObject>]>;

    /// Get objects return an array of optinal mutable references to the PDO objects
    ///
    /// This method is commonly derived using the [`ethercat_hal_derive::TxPdo`] macro.
    fn get_objects_mut(&mut self) -> Box<[Option<&mut dyn TxPdoObject>]>;

    /// Calculating the size of the PDO assignment in bits
    ///
    /// Only the PDO objects that are Some(_) are counted.
    ///
    /// Will always be rouneded to the next byte since not all bits are always used but space in the PDU is reserved on byte basis.
    fn size(&self) -> usize {
        let used_bits = self
            .get_objects()
            .iter()
            .map(|object| match object {
                Some(object) => object.size(),
                None => 0,
            })
            .sum::<usize>();
        let padding = match used_bits % 8 {
            0 => 0,
            _ => 8 - used_bits % 8,
        };
        used_bits + padding
    }

    /// Will give the PDU bit array to the PDO objects to decode the data
    fn read(&mut self, buffer: &BitSlice<u8, Lsb0>) -> Result<(), anyhow::Error> {
        let mut bit_offset = 0;
        for object in self.get_objects_mut().iter_mut() {
            if let Some(object) = object {
                let end_bit_index = bit_offset + object.size();

                // check if end_bit_index is out of bounds
                if end_bit_index > buffer.len() {
                    return Err(anyhow::anyhow!(
                        "[{}::RxPdo::write] Range {}..{} ({}bits) is out of bounds for buffer with length {}",
                        module_path!(),
                        bit_offset,
                        end_bit_index,
                        object.size(),
                        buffer.len()
                    ));
                }

                object.read(&buffer[bit_offset..end_bit_index]);
                bit_offset += object.size();
            }
        }
        Ok(())
    }
}
