use crate::coe::Configuration;
use bitvec::prelude::*;

pub mod basic;
pub mod el252x;
pub mod el30xx;
pub mod el32xx;

/// This trait allows to know the size of a PDO object in bits.
///
/// Can be derived with the [`ethercat_hal_derive::PdoObject`] macro.
///
/// Example:
/// ```rust,no_run
/// #[derive(PdoObject)]
/// #[pdo_object(bits = 1)]
/// struct BoolPdoObject {
///    pub value: bool,
/// }
/// ````
///
/// Equivalent:
/// ```rust,no_run
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
/// ```rust,no_run
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
/// ```rust,no_run
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
/// ```rust,no_run
/// #[derive(Debug, Clone)]
/// pub enum EL3001PdoPreset {
///     Standard,
///     Compact,
/// }
///
/// impl PredefinedPdoAssignment<EL3001TxPdo, EL3001RxPdo> for EL3001PdoPreset {
///     fn txpdo_assignment(&self) -> EL3001TxPdo {
///         match self {
///             EL3001PdoPreset::Standard => EL3001TxPdo {
///                 ai_standard: Some(AiStandard::default()),
///                 ai_compact: None,
///             },
///             EL3001PdoPreset::Compact => EL3001TxPdo {
///                 ai_standard: None,
///                 ai_compact: Some(AiCompact::default()),
///             },
///         }
///     }
///
///     fn rxpdo_assignment(&self) -> EL3001RxPdo {///
///        match self {
///            EL3001PdoPreset::Standard => EL3001RxPdo {},
///            EL3001PdoPreset::Compact => EL3001RxPdo {},
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
/// Can be derived using the [`ethercat_hal_derive::TxPdo`] macro with the `#[pdo_object_index]` attribute.
///
/// ```rust,no_run
/// #[derive(Debug, Clone, TxPdo)]
/// pub struct EL1002TxPdo {
///     #[pdo_object_index(0x1600)]
///     pub channel1: Option<BoolPdoObject>,
///     #[pdo_object_index(0x1601)]
///     pub channel2: Option<BoolPdoObject>,
/// }
/// ```
///
/// Since it bounds to `Configuration` it can read the PDO assignment from the device using the [`TxPdo::read`] method.
pub trait TxPdo: Configuration {
    /// Get objects return an array of optional references to the PDO objects
    ///
    /// This method is commonly derived using the [`ethercat_hal_derive::TxPdo`] macro.
    fn get_objects(&self) -> &[Option<&dyn TxPdoObject>];

    /// Get mutable objects return an array of optional mutable references to the PDO objects
    ///
    /// This method is commonly derived using the [`ethercat_hal_derive::TxPdo`] macro.
    fn get_objects_mut(&mut self) -> &mut [Option<&mut dyn TxPdoObject>];

    /// Calculating the size of the PDO assignment in bits
    ///
    /// Only the PDO objects that are Some(_) are counted.
    ///
    /// Will always be rounded to the next byte since not all bits are always used but space in the PDU is reserved on byte basis.
    fn size(&self) -> usize {
        let used_bits = self
            .get_objects()
            .iter()
            .map(|objects| objects.map(|object| object.size()).unwrap_or(0))
            .sum::<usize>();
        let padding = match used_bits % 8 {
            0 => 0,
            _ => 8 - used_bits % 8,
        };
        return used_bits + padding;
    }

    /// Reading the PDO assignment from the device
    fn read(&mut self, buffer: &BitSlice<u8, Lsb0>) {
        let mut bit_offset = 0;
        for object in self.get_objects_mut().iter_mut() {
            if let Some(object) = object {
                let end_bit_index = bit_offset + object.size();
                object.read(&buffer[bit_offset..end_bit_index]);
                bit_offset += object.size();
            }
        }
    }
}
