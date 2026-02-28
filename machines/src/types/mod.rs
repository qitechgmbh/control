use serde::{Deserialize, Serialize};

#[derive(Debug,Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Direction
{
    Forward,
    Reverse,
}

impl Direction
{
    pub fn from_bool(forward: bool) -> Self
    {
        match forward 
        {
            true  => Self::Forward,
            false => Self::Reverse,
        }
    }

    pub fn is_forward(self) -> bool
    {
        self == Self::Forward
    }

    pub fn is_reverse(self) -> bool
    {
        self != Self::Forward
    }
}