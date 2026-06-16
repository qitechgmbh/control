use std::fmt::Debug;

use serde::{Serialize, Serializer, ser::{SerializeSeq, SerializeStruct}};

use crate::{
    POOL_CAPACITY_MAX_BOOLEAN, POOL_CAPACITY_MAX_FLOAT, POOL_CAPACITY_MAX_INTEGER,
    POOL_CAPACITY_MAX_STRING, PropertyPool, PropertySet, StringPropertyValue,
};

pub struct DirtyPropertySetExportView<'a> {
    float: DirtyPropertyPoolExportView<'a, f64, POOL_CAPACITY_MAX_FLOAT>,
    int: DirtyPropertyPoolExportView<'a, i64, POOL_CAPACITY_MAX_INTEGER>,
    r#bool: DirtyPropertyPoolExportView<'a, bool, POOL_CAPACITY_MAX_BOOLEAN>,
    string: DirtyPropertyPoolExportView<'a, StringPropertyValue, POOL_CAPACITY_MAX_STRING>,
}

impl<'a> From<&'a PropertySet> for DirtyPropertySetExportView<'a> {
    fn from(value: &'a PropertySet) -> Self {
        Self {
            float: DirtyPropertyPoolExportView { 
                pool: &value.float 
            },
            int: DirtyPropertyPoolExportView { 
                pool: &value.int 
            },
            r#bool: DirtyPropertyPoolExportView {
                pool: &value.r#bool,
            },
            string: DirtyPropertyPoolExportView {
                pool: &value.string,
            },
        }
    }
}

impl<'a> Serialize for DirtyPropertySetExportView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("DirtyPropertySetExporter", 4)?;
        state.serialize_field("float", &self.float)?;
        state.serialize_field("int", &self.int)?;
        state.serialize_field("bool", &self.r#bool)?;
        state.serialize_field("string", &self.string)?;
        state.end()
    }
}

pub struct DirtyPropertyPoolExportView<'a, T: Default + Debug, const CAPACITY: usize> {
    pool: &'a PropertyPool<T, CAPACITY>,
}

impl<'a, T, const CAPACITY: usize> Serialize for DirtyPropertyPoolExportView<'a, T, CAPACITY>
where
    T: Default + Debug + serde::Serialize + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;

        for entry in self.pool.iter_dirty() {
            seq.serialize_element(entry)?;
        }

        seq.end()
    }
}

#[cfg(test)]
mod test {
    use crate::{DirtyPropertySetExportView, ExportedPropertySet, PropertyEntry, PropertySet};

    #[test]
    pub fn serde_roundtrip() {
        let mut set = PropertySet::default();
        _ = set.float.add(PropertyEntry {
            ident: 0,
            name: "hello.world",
            value: 10.0,
        });

        let exporter = DirtyPropertySetExportView::from(&set);

        let encoded = serde_json::to_string_pretty(&exporter).unwrap();
        println!("encoded: {}", encoded);

        let decoded = serde_json::from_str::<ExportedPropertySet>(&encoded).unwrap();
        println!("decoded: {:?}", decoded);
    }
}