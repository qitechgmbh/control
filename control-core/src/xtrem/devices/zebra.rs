use crate::xtrem::{devices::{Device, HandleResponseError, Priority, Scheduler}, protocol::{DataAddress, Request, Response}};

#[derive(Debug)]
pub struct XtremZebra<S: Scheduler> {
    scheduler: S,
    weight: Option<f64>,
}

impl<S: Scheduler> XtremZebra<S> {
    pub fn sync_weight(&mut self) {
        self.scheduler.schedule(Priority::Medium);
    }

    pub fn weight(&mut self) -> Option<f64> {
        self.weight
    }
}

impl<S: Scheduler> Device<S> for XtremZebra<S> {
    fn new(scheduler: S) -> Self where Self: Sized {
        Self { scheduler, weight: None }
    }

    fn next_request(&mut self) -> Option<(Request, bool)> {
        let request = Request {
            address: todo!(),
            function: todo!(),
        };
        let has_next = false;
        Some((request, has_next))
    }

    fn handle_response(&mut self, response: Response) -> Result<(), HandleResponseError> {
        use Response::*;
        match response {
            Read(data) => {

            },
            _ => return Err(HandleResponseError::InvalidFunction),
        }
    }
}