use std::time::{Duration, Instant};

use anyhow::anyhow;
use chrono::{Datelike, Local, Timelike};
use stahlwerk_extension::{Date, Time};

use stahlwerk_extension::ff01::{
    Entry, 
    FinalizeRequest, 
    ProxyClient, 
    ProxyTransactionError, 
    Request, 
    Response
};

mod types;
use types::State;

use crate::stahlwerk::ff01::workorder_service::types::{StateOneData, StateTwoData};

#[derive(Debug)]
pub struct WorkorderService 
{
    // config / dependencies
    enabled: bool,
    client: Option<ProxyClient>,
    request_timeout: Duration,

    state: State,
    last_request_ts: Instant,
}

// public interface
impl WorkorderService 
{
    pub fn new(request_timeout: Duration) -> Self {
        Self { 
            enabled: false, 
            client: None, 
            state: State::Zero,
            request_timeout, 
            last_request_ts: Instant::now(), 
        }
    }

    pub fn set_enabled(&mut self, value: bool) {
        if self.enabled == value { return; }
        self.enabled = value;
    }

    pub fn connect(&mut self, config_path: &str) -> anyhow::Result<()> {
        use stahlwerk_extension::ff01::ProxyClient;
        use stahlwerk_extension::ClientConfig;

        if self.client.is_some() {
            return Ok(());
        }

        self.state = State::Zero;

        let config = ClientConfig::from_file(config_path)
            .map_err(|e| anyhow!("[FF01] Failed to read Config: {:?}", e))?;

        let client = ProxyClient::new(config)
            .map_err(|e| anyhow!("[FF01] Failed to create Client: {:?}", e))?;
   
        self.client = Some(client);
        return Ok(())
    }

    pub fn disconnect(&mut self) {
        self.state = State::Zero;
        self.client = None;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    pub fn update_recv(&mut self) -> anyhow::Result<()> {
        let Some(mut client) = self.client.take() else {
            return Ok(());
        };

        if !client.has_pending_request() {
            return Ok(());
        } 

        let maybe_response = self.poll_response(&mut client)?;
        self.client = Some(client);

        let Some(response) = maybe_response else {
            return Ok(());
        };

        self.handle_response(response)?;
        return Ok(());
    }

    pub fn update_send(&mut self, now: Instant, plates_counted: u32) -> anyhow::Result<()>  {
        let Some(mut client) = self.client.take() else {
            return Ok(());
        };

        if client.has_pending_request() {
            return Ok(());
        } 

        self.send_next_request(now, &mut client, plates_counted);
        self.client = Some(client);
        Ok(())
    }

    pub fn current_entry(&self) -> Option<&Entry> {
        match &self.state {
            State::Zero => None,
            State::One(data) => Some(&data.entry),
            State::Two(data) => Some(&data.state_one_data.entry),
        }
    }
}

// utils
impl WorkorderService 
{
    fn send_next_request(&mut self, now: Instant, client: &mut ProxyClient, quantity_counted: u32) {
        use State::*;

        let request = match &self.state {
            Zero => Request::GetNextEntry,
            One(data) => Request::GetWorkerSubmission(&data.entry),
            Two(data2) => {
                let data1 = &data2.state_one_data;

                let doc_entry = data1.entry.doc_entry;

                let start_date = data1.start_date;
                let from_time = data1.from_time;

                let personnel_id = data2.personnel_id.clone();
                let quantity_scrap = data2.quantity_scrap;

                let chrono_now = Local::now();
                let end_date = Date { year: chrono_now.year(), month: chrono_now.month(), day: chrono_now.day() };
                let to_time = Time { hour: chrono_now.hour(), minute: chrono_now.minute() };

                let request_data = FinalizeRequest {
                    doc_entry,
                    personnel_id,
                    start_date,
                    end_date,
                    from_time,
                    to_time,
                    quantity_scrap,
                    quantity_counted,
                };

                Request::Finalize(request_data)
            },
        };

        if self.last_request_ts + self.request_timeout < now {
            // timeout nor reached, can'T send request yet
            return;
        }

        client.queue_request(request).expect("Should be able to enqueue");
        self.last_request_ts = now;
    }

    fn handle_response(&mut self, response: Response) -> anyhow::Result<()> {
        use State::*;

        let current_state = std::mem::take(&mut self.state);

        self.state = match current_state {
            Zero => {
                let Response::GetNextEntry(maybe_entry) = response else {
                    return Err(anyhow!("Tag Mismatch"));
                };  

                let Some(entry) = maybe_entry else {
                    // no entry found
                    return Ok(());
                };

                let now = Local::now();
                let start_date = Date { year: now.year(), month: now.month(), day: now.day() };
                let from_time = Time { hour: now.hour(), minute: now.minute() };

                let data = StateOneData { entry, start_date, from_time };

                State::One(data)
            },
            One(state_one_data) => {
                let Response::GetWorkerSubmission(maybe_workorder_submission) = response else {
                    return Err(anyhow!("Tag Mismatch"));
                };

                let Some((personnel_id, quantity_scrap)) = maybe_workorder_submission else {
                    // no entry found
                    return Ok(());
                };

                let data = StateTwoData { state_one_data, personnel_id, quantity_scrap };
                State::Two(data)
            },
            Two(state_two_data) => {
                _ = state_two_data;

                let Response::Finalize = response else {
                    return Err(anyhow!("Tag Mismatch"));
                };

                State::Zero
            },
        };

        Ok(())
    }

    fn poll_response(&mut self, client: &mut ProxyClient) -> anyhow::Result<Option<Response>> {

        let result = client.poll_response();

        match result {
            Ok(v) => Ok(Some(v)),
            Err(e) => {
                match e {
                    ProxyTransactionError::Pending => Ok(None),
                    e => Err(anyhow!("PollResponseErr: {:?}", e)) 
                }
            },
        }
    }
}