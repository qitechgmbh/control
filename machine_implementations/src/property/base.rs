use std::marker::PhantomData;
use super::STRING_LEN_MAX;

#[derive(Debug)]
pub struct SimpleProperty<T> {
    dirty: *mut bool,
    value: *mut T,
}

impl<T: Copy> SimpleProperty<T> {
    pub fn new(dirty: *mut bool, value: *mut T) -> Self {
       Self { dirty, value }
    }

    pub fn get(&self) -> T {
       unsafe { *self.value }
    }

    pub fn set(&mut self, value: T) {
        unsafe {
            *self.value = value;
            *self.dirty = true;
        }
    }

    pub fn is_dirty(&self) -> bool {
        unsafe { *self.dirty }
    }
}

#[derive(Debug)]
pub struct EnumProperty<T: Copy + From<i64> + Into<i64>> {
    dirty: *mut bool,
    value: *mut i64,
    phantom_data: PhantomData<T>,
}

impl<T: Copy + From<i64> + Into<i64>> EnumProperty<T> {
    pub fn get(&self) -> i64 {
       unsafe { *self.value }
    }

    pub fn set(&mut self, value: T) {
        unsafe {
            *self.value = value.into();
            *self.dirty = true;
        }
    }
}

#[derive(Debug)]
pub struct StringProperty {
    dirty: *mut bool,
    value: *mut heapless::String<STRING_LEN_MAX>,
}

impl StringProperty {
    pub fn get(&self) -> &heapless::String<STRING_LEN_MAX> {
       unsafe { &*self.value }
    }

    pub fn set(&mut self, value: heapless::String<STRING_LEN_MAX>) {
        unsafe {
            *self.value = value;
            *self.dirty = true;
        }
    }
}

