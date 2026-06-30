use std::fmt::Debug;

use super::StringPropertyValue;

type Buffer<T> = Vec<ExportedPropertyEntry<T>>;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct ExportedPropertyEntry<T: Debug> {
    pub ident: u64,
    pub name: String,
    pub value: T,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct ExportedPropertySet {
    pub float: Buffer<f64>,
    pub int: Buffer<i64>,
    pub r#bool: Buffer<bool>,
    pub string: Buffer<StringPropertyValue>,
}
