use std::slice::IterMut;
use qitech_lib::machines::MachineIdentificationUnique;
use crate::{property::PropertyEntry};

pub struct IterItem<T> {
    pub ident: MachineIdentificationUnique,
    pub name: &'static str,
    pub value: T,
}

pub struct Iter<'a, T> {
    inner: IterMut<'a, PropertyEntry<T>>,
    index: usize,
}

impl<'a, T> Iter<'a, T> {
    pub(super) fn new(inner: IterMut<'a, PropertyEntry<T>>) -> Self {
        Self { inner, index: 0 }
    }
}

impl<'a, T: Copy> Iterator for Iter<'a, T> {
    type Item = IterItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        for entry in self.inner.by_ref() {
            self.index += 1;

            if entry.dirty {
                entry.dirty = false;
                return Some(IterItem { 
                    ident: entry.ident, 
                    name: entry.name, 
                    value: entry.value
                });
            }
        }

        None
    }
}
