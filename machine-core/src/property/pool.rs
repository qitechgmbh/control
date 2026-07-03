use std::fmt::Debug;
use super::{PropertyAllocatorError, PropertyEntry};

#[derive(Debug, Default, Clone)]
pub struct PropertySlot<T: Clone> {
    entry: PropertyEntry<T>,
    dirty: bool,
    default_value: T,
    can_reset_dirty: bool,
    can_reset_value: bool,
}

impl<T: Clone> PropertySlot<T>  {
    pub fn new(    
        entry: PropertyEntry<T>,
        dirty: bool,
        default_value: T,
        can_reset_dirty: bool,
        can_reset_value: bool
    ) -> Self {
        Self { entry, dirty, default_value, can_reset_dirty, can_reset_value }
    }

    fn reset(&mut self) {
        if self.can_reset_dirty {
            self.dirty = false;
        }

        if self.can_reset_value {
            self.entry.value = self.default_value.clone();
        }
    }
}

#[derive(Clone)]
pub struct PropertyPool<T: Clone + Default, const CAPACITY: usize> {
    slots: [Option<PropertySlot<T>>; CAPACITY],
    last: usize,
}

impl<T: Clone + Default, const CAPACITY: usize> PropertyPool<T, CAPACITY> {
    pub fn add(&mut self, slot: PropertySlot<T>) -> Result<(*mut T, *mut bool), PropertyAllocatorError> {
        // try to find empty slot
        for i in 0..self.last {
            if self.slots[i].is_none() {
                self.slots[i] = Some(slot);
                let slot_ref = self.slots[i].as_mut().expect("Just inserted");

                return Ok((
                    &mut slot_ref.entry.value,
                    &mut slot_ref.dirty,
                ));
            }
        }

        // append if space available
        if self.last < CAPACITY {
            let idx = self.last;
            self.slots[idx] = Some(slot);
            self.last += 1;
            
            let slot_ref = self.slots[idx].as_mut().expect("Just inserted");

            return Ok((
                &mut slot_ref.entry.value,
                &mut slot_ref.dirty,
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
        self.slots[index] = None;

        // if removing the last active element, shrink `last`
        if index == self.last.saturating_sub(1) {
            self.last = self.last.saturating_sub(1);

            // walk backwards to find new last valid entry
            while self.last > 0 {
                let i = self.last - 1;

                if self.slots[i].is_some() {
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
            let Some(slot) = item else { continue };
            if slot.dirty { count += 1 }
        }

        count
    }

    pub fn reset_properties(&mut self) {
        for item in &mut self.slots {
            let Some(slot) = item else { continue };
            slot.reset();
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

impl<T: Clone + Default, const CAPACITY: usize> Default for PropertyPool<T, CAPACITY> {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|_| Default::default()),
            last: 0,
        }
    }
}

pub struct PropertyPoolIter<'a, T: Clone> {
    items: &'a [Option<PropertySlot<T>>],
    last: usize,
    index: usize,
    dirty_only: bool,
}

impl<'a, T: Clone> Iterator for PropertyPoolIter<'a, T>
where
    T: Debug,
{
    type Item = &'a PropertyEntry<T>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.last {
            let item = &self.items[self.index];
            self.index += 1;

            let Some(slot) = item else {
                continue;
            };

            if self.dirty_only && !slot.dirty {
                continue;
            }

            return Some(&slot.entry);
        }

        None
    }
}
