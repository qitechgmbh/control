#[derive(Debug, Clone)]
pub enum Signal {
    /// doens't start any more tasks
    Exit,
    /// continues with the current number of concurrent tasks
    Continue,
}
