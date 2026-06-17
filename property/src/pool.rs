use std::fmt::Debug;
use crate::{AllocatorError, PropertyEntry};

#[derive(Debug, Default, Clone)]
pub struct PropertySlot<T: Default + Debug> {
    pub entry: Option<PropertyEntry<T>>,
    pub dirty: bool,
}

#[derive(Debug, Clone)]
pub struct PropertyPool<T: Debug + Default, const CAPACITY: usize> {
    slots: [PropertySlot<T>; CAPACITY],
    last: usize,
}

impl<T: Debug + Default, const CAPACITY: usize> PropertyPool<T, CAPACITY> {
    pub fn add(&mut self, entry: PropertyEntry<T>) -> Result<(*mut T, *mut bool), AllocatorError> {
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
            self.last += 1;

            return Ok((
                &mut self.slots[idx].entry.as_mut().expect("must be Some").value,
                &mut self.slots[idx].dirty,
            ));
        }

        // no space remaining
        Err(AllocatorError)
    }

    pub fn remove(&mut self, index: usize) -> Result<(), AllocatorError> {
        if index >= self.last {
            return Err(AllocatorError);
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

    pub fn dirty_count(&self) -> usize {
        let mut count = 0;
        for item in &self.slots {
            if item.dirty {
                count += 1;
            }
        }

        count
    }

    pub fn clear_dirty_flags(&mut self) {
        for item in &mut self.slots {
            item.dirty = false;
        }
    }

    pub fn iter_dirty(&self) -> BufferDirtyIter<'_, T, CAPACITY> {
        BufferDirtyIter {
            items: &self.slots,
            index: 0,
            last: self.last,
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

pub struct BufferDirtyIter<'a, T: Debug + Default, const CAPACITY: usize> {
    items: &'a [PropertySlot<T>; CAPACITY],
    last: usize,
    index: usize,
}

impl<'a, T, const CAPACITY: usize> Iterator for BufferDirtyIter<'a, T, CAPACITY>
where
    T: Debug + Default,
{
    type Item = &'a PropertyEntry<T>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.last {
            let item = &self.items[self.index];
            self.index += 1;

            if !item.dirty {
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
