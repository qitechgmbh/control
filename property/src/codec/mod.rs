use std::fmt::Debug;
use serde::Deserialize;
use serde::Serialize;

use crate::StringPropertyValue;

mod export_view;
pub use export_view::DirtyPropertySetExportView;
pub use export_view::DirtyPropertyPoolExportView;

type Buffer<T> = Vec<ExportedPropertyEntry<T>>;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "clickhouse", derive(clickhouse::Row))]
pub struct ExportedPropertyEntry<T: Default + Debug> {
    pub ident: u64,
    pub name: String,
    pub value: T,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportedPropertySet {
    pub float: Buffer<f64>,
    pub int: Buffer<i64>,
    pub r#bool: Buffer<bool>,
    pub string: Buffer<StringPropertyValue>,
}
