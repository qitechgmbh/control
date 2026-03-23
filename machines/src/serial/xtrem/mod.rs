#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority
{
    Low,
    Medium,
    High,
    Urgent,
}

pub trait Scheduler 
{
    /// notifies manager that this device
    /// want's to submit a request with
    /// certain priority. 
    /// Operation should not fail.
    fn schedule(&self, priority: Priority);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError
{
    /// (index: u16, received: u16)
    InvalidValue(u16, u16),

    /// (minimum: u16, received: u16)
    DataTooSmall(u16, u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleResponseError
{
    ParseError(ParseError),
    InvalidFunctionCode(FunctionCode),
    NoResponseExpected,
}

pub trait Device<S: Scheduler>
{
    fn new(scheduler: S) -> Self where Self: Sized;

    fn next_request(&mut self) -> Option<(Request, bool)>;

    fn handle_response(&mut self, response: Response) -> Result<(), HandleResponseError>;
}