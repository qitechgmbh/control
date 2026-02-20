use crate::stahlwerk::{ r#abstract::BaseMachine };

pub mod common;
pub mod api;
pub mod act;

pub(super) use TemplateMachine as DerivedMachine;

#[derive(Debug)]
pub struct TemplateMachine
{
    base: BaseMachine,
}

impl TemplateMachine
{
    pub fn new(base: BaseMachine) -> Self
    {
        Self { base }
    }
}