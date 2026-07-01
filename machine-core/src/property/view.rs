use std::fmt::Debug;

use super::{
    ExportedPropertyEntry, ExportedPropertySet, PropertySet,
    pool::{PropertyPoolDirtyIter, PropertyPoolIter},
};

pub struct PropertySetView<'a> {
    pub float: PropertyBufferIter<'a, f64>,
    pub integer: PropertyBufferIter<'a, i64>,
}

impl<'a> PropertySetView<'a> {
    pub fn native(value: &'a PropertySet) -> Self {
        Self {
            float: PropertyBufferIter::Native(value.float.iter()),
            integer: PropertyBufferIter::Native(value.integer.iter()),
        }
    }

    pub fn native_dirty(value: &'a PropertySet) -> Self {
        Self {
            float: PropertyBufferIter::NativeDirty(value.float.iter_dirty()),
            integer: PropertyBufferIter::NativeDirty(value.integer.iter_dirty()),
        }
    }

    pub fn exported(value: &'a ExportedPropertySet) -> Self {
        Self {
            float: PropertyBufferIter::Exported(value.float.iter()),
            integer: PropertyBufferIter::Exported(value.int.iter()),
        }
    }
}

#[derive(Debug)]
pub struct PropertyView<'a, T> {
    pub ident: u64,
    pub name: &'a str,
    pub value: &'a T,
}

pub enum PropertyBufferIter<'a, T: Debug> {
    Native(PropertyPoolIter<'a, T>),
    NativeDirty(PropertyPoolDirtyIter<'a, T>),
    Exported(std::slice::Iter<'a, ExportedPropertyEntry<T>>),
}

impl<'a, T: Debug> Iterator for PropertyBufferIter<'a, T> {
    type Item = PropertyView<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            PropertyBufferIter::Native(iter) => {
                let entry = iter.next()?;
                Some(PropertyView {
                    ident: entry.ident,
                    name: entry.name,
                    value: &entry.value,
                })
            }

            PropertyBufferIter::NativeDirty(iter) => {
                let entry = iter.next()?;
                Some(PropertyView {
                    ident: entry.ident,
                    name: entry.name,
                    value: &entry.value,
                })
            }

            PropertyBufferIter::Exported(iter) => {
                let entry = iter.next()?;
                Some(PropertyView {
                    ident: entry.ident,
                    name: &entry.name,
                    value: &entry.value,
                })
            }
        }
    }
}
