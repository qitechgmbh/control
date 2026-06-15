use super::{
    STRING_LEN_MAX,
    POOL_CAPACITY_MAX_FLOAT,
    POOL_CAPACITY_MAX_INTEGER,
    POOL_CAPACITY_MAX_BOOLEAN,
    POOL_CAPACITY_MAX_STRING,
    Iter,
    PropertyEntry,
};

// must be heapless::Vec NOT Vec so the pointers 
// aren't invalided on resize since we can't resize.
type PropertyBuffer<T, const CAPACITY: usize> = heapless::Vec<PropertyEntry<T>, CAPACITY>;
type String = heapless::String<STRING_LEN_MAX>;

type PropertyBufferFloat = PropertyBuffer<f64, POOL_CAPACITY_MAX_FLOAT>;
type PropertyBufferInt = PropertyBuffer<i64, POOL_CAPACITY_MAX_INTEGER>;
type PropertyBufferBool = PropertyBuffer<bool, POOL_CAPACITY_MAX_BOOLEAN>;
type PropertyBufferString = PropertyBuffer<String, POOL_CAPACITY_MAX_STRING>;

#[derive(Debug, Default)]
pub struct Pool {
    pub(super) buf_float: PropertyBufferFloat,
    pub(super) buf_int: PropertyBufferInt,
    pub(super) buf_bool: PropertyBufferBool,
    pub(super) buf_string: PropertyBufferString,
}

impl Pool {
    pub fn consume_buffer_float(&'_ mut self) -> Iter<'_, f64> {
        Iter::new(self.buf_float.iter_mut())
    }

    pub fn consume_buffer_int(&'_ mut self) -> Iter<'_, i64> {
        Iter::new(self.buf_int.iter_mut())
    }

    pub fn consume_buffer_bool(&'_ mut self) -> Iter<'_, bool> {
        Iter::new(self.buf_bool.iter_mut())
    }

    pub fn consume_buffer_string(&'_ mut self) -> Iter<'_, String> {
        Iter::new(self.buf_string.iter_mut())
    }
}
