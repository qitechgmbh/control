use serde::{Deserialize, Serialize};

#[derive(Debug,Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Direction
{
    Forward,
    Reverse,
}