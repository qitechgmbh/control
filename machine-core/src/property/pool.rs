use std::fmt::Debug;
use super::{PropertyAllocatorError, PropertyEntry};

#[derive(Debug, Default, Clone)]
struct PropertySlot<T> {
    entry: Option<PropertyEntry<T>>,
    dirty: bool,
    dirty_always: bool,
}

#[derive(Clone)]
pub struct PropertyPool<T: Default, const CAPACITY: usize> {
    slots: [PropertySlot<T>; CAPACITY],
    last: usize,
}

impl<T: Debug + Default, const CAPACITY: usize> PropertyPool<T, CAPACITY> {
    pub fn add(&mut self, entry: PropertyEntry<T>, always_dirty: bool) -> Result<(*mut T, *mut bool), PropertyAllocatorError> {
        // try to find empty slot
        for i in 0..self.last {
            if self.slots[i].entry.is_none() {
                self.slots[i].entry = Some(entry);
                self.slots[i].dirty = true;

                return Ok((
                    &mut self.slots[i].entry.as_mut().expect("must be Some").value,
                    &mut self.slots[i].dirty,
                ));
            }
        }

        // append if space available
        if self.last < CAPACITY {
            let idx = self.last;
            self.slots[idx].entry = Some(entry);
            self.slots[idx].dirty = true;
            self.slots[idx].dirty_always = always_dirty;
            self.last += 1;

            return Ok((
                &mut self.slots[idx].entry.as_mut().expect("must be Some").value,
                &mut self.slots[idx].dirty,
            ));
        }

        // no space remaining
        Err(PropertyAllocatorError)
    }

    pub fn remove(&mut self, index: usize) -> Result<(), PropertyAllocatorError> {
        if index >= self.last {
            return Err(PropertyAllocatorError);
        }

        // mark as unused
        self.slots[index].entry = None;
        self.slots[index].dirty = false;

        // if removing the last active element, shrink `last`
        if index == self.last.saturating_sub(1) {
            self.last = self.last.saturating_sub(1);

            // walk backwards to find new last valid entry
            while self.last > 0 {
                let i = self.last - 1;

                if self.slots[i].entry.is_some() {
                    self.last = i;
                    break;
                }

                self.last -= 1;
            }
        }

        Ok(())
    }

    pub fn remove_by_ident(&mut self, ident: u64) {
        //TODO: implement
        _ = ident;
    }

    pub fn dirty_count(&self) -> usize {
        let mut count = 0;
        for item in &self.slots {
            if item.dirty {
                count += 1;
            }
        }

        count
    }

    pub fn reset_dirty_flags(&mut self) {
        for item in &mut self.slots {
            item.dirty = false;
        }
    }

    pub fn iter(&self, dirty_only: bool) -> PropertyPoolIter<'_, T> {
        PropertyPoolIter {
            items: &self.slots,
            index: 0,
            last: self.last,
            dirty_only,
        }
    }
}

impl<T: Debug + Default, const CAPACITY: usize> Default for PropertyPool<T, CAPACITY> {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|_| PropertySlot::default()),
            last: 0,
        }
    }
}

pub struct PropertyPoolIter<'a, T: Debug> {
    items: &'a [PropertySlot<T>],
    last: usize,
    index: usize,
    dirty_only: bool,
}

impl<'a, T> Iterator for PropertyPoolIter<'a, T>
where
    T: Debug,
{
    type Item = &'a PropertyEntry<T>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.last {
            let item = &self.items[self.index];
            self.index += 1;

            if self.dirty_only && !item.dirty {
                continue;
            }

            let Some(entry) = &item.entry else {
                continue;
            };

            return Some(entry);
        }

        None
    }
}
