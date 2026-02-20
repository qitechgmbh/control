#[derive(Debug, Clone)]
pub struct MachineId
{
    pub category_id: u8,
    pub index:       u8,
    pub slug:        &'static str,
}

impl MachineId 
{
    pub const fn new(category_id: u8, index: u8, slug: &'static str) -> Self 
    {
        Self { category_id, index, slug }
    }

    pub const fn to_u16(&self) -> u16 
    {
        ((self.category_id as u16) << 8) | (self.index as u16)
    }
}