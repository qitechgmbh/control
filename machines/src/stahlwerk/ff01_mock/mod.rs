use crate::stahlwerk::{ r#abstract::BaseMachine };

pub mod common;
pub mod api;
pub mod act;

pub(super) use FF01MachineMock as DerivedMachine;

#[derive(Debug)]
pub struct FF01MachineMock
{
    base: BaseMachine,

    counter: u64,

    // todo: current processed entry HERE
}

impl FF01MachineMock
{
    pub fn new(base: BaseMachine) -> Self
    {
        Self { base, counter: 0 }
    }

    pub fn increment(&mut self)
    {
        self.counter += 1;
        self.emit_state();
    }

    pub fn finalize(&mut self)
    {
        todo!("Submit backflush")
        // self.emit_state();
    }
}