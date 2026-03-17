use std::time::{Duration, Instant};

use anyhow::anyhow;
use chrono::{Datelike, Local, Timelike};
use stahlwerk_extension::{Date, Time, ff01::{Entry, FinalizeRequest, ProxyClient, Request, Response}};

#[derive(Debug)]
pub struct WorkorderService 
{
    // config / dependencies
    client: ProxyClient,
    request_timeout: Duration,

    // state
    entry: Option<Entry>,
    plates_counted: u32,
    last_request_ts: Instant,

    // quantity_scrap, personnel_id
    worker_submission: Option<(String, f64)>,

    start_date: Date,
    from_time:  Time,
}

// public interface
impl WorkorderService 
{
    pub fn new(client: ProxyClient, request_timeout: Duration) -> Self {
        Self { 
            client, 
            request_timeout, 
            entry: None, 
            plates_counted: 0, 
            last_request_ts: Instant::now(),
            worker_submission: None,

            start_date: Date { year: 0, month: 0, day: 0 },
            from_time: Time { hour: 0, minute: 0 },
        }
    }   

    pub fn submit_plate(&mut self, weight: f64) -> PlateSubmitResult {
        let Some(entry) = &self.entry else {
            return PlateSubmitResult::NotCounting;
        };

        if !entry.weight_bounds.in_bounds(weight) {
            return PlateSubmitResult::OutOufBOunds;
        }

        self.plates_counted += 1;
        PlateSubmitResult::InBounds
    }

    pub fn update(&mut self, now: Instant) -> anyhow::Result<()> {

        let was_none = self.entry.is_none();

        let Some(entry) = self.get_entry(now)? else {
            return Ok(());
        };

        if was_none {
            let now = Local::now();
            self.start_date = Date { year: now.year(), month: now.month(), day: now.day() };
            self.from_time = Time { hour: now.hour(), minute: now.minute() };
        }

        let Some((personnel_id, quantity_scrap)) = self.get_submission(now, &entry)? else {
            return Ok(());
        };

        let finished = self.finalize_workorder(now, &entry, &personnel_id, quantity_scrap)?;

        if finished {
            self.entry = None;
            self.worker_submission = None;
            return Ok(());
        }

        self.entry = Some(entry);
        self.worker_submission = Some((personnel_id, quantity_scrap));
        Ok(())
    }

    pub fn plates_counted(&self) -> u32 {
        self.plates_counted
    }

    pub fn current_entry(&self) -> &Option<Entry> {
        &self.entry
    }
}

// utils
impl WorkorderService 
{
    fn get_entry(&mut self, now: Instant) -> anyhow::Result<Option<Entry>>
    {
        if let Some(entry) = self.entry.take() {
            return Ok(Some(entry));
        }

        self.fetch_next_entry(now)
    }

    fn get_submission(&mut self, now: Instant, entry: &Entry) -> anyhow::Result<Option<(String, f64)>>
    {
        if let Some(v) = self.worker_submission.take() {
            return Ok(Some(v));
        }

        self.fetch_worker_submission(now, entry)
    }

    fn fetch_next_entry(&mut self, now: Instant) -> anyhow::Result<Option<Entry>> {
        // awaiting pending request
        if self.client.has_pending_request() {
            if let Response::GetNextEntry(entry) = self.poll_response()? {
                return Ok(entry);
            } 

            return Err(anyhow!("Tag Mismatch"));
        }

        // no pending requests, so submit new request
        let request = Request::GetNextEntry;
        self.submit_request(now, request);
        Ok(None)
    }

    fn fetch_worker_submission(&mut self, now: Instant, entry: &Entry) -> anyhow::Result<Option<(String, f64)>> {
        // awaiting pending request
        if self.client.has_pending_request() {
            if let Response::GetWorkerSubmission(workorder_submission) = self.poll_response()? {
                return Ok(workorder_submission);
            } 

            return Err(anyhow!("Tag Mismatch"));
        }

        // no pending requests, so submit new request
        let request = Request::GetWorkerSubmission(entry);
        self.submit_request(now, request);
        Ok(None)
    }

    fn finalize_workorder(&mut self, now: Instant, entry: &Entry, personnel_id: &String, quantity_scrap: f64) -> anyhow::Result<bool> {
        // awaiting pending request
        if self.client.has_pending_request() {
            if let Response::Finalize = self.poll_response()? {
                return Ok(true);
            } 

            return Err(anyhow!("Tag Mismatch"));
        }

        let chrono_now = Local::now();
        let end_date = Date { year: chrono_now.year(), month: chrono_now.month(), day: chrono_now.day() };
        let to_time = Time { hour: chrono_now.hour(), minute: chrono_now.minute() };

        let request_data = FinalizeRequest {
            doc_entry: entry.doc_entry,
            personnel_id: personnel_id.clone(),
            start_date: self.start_date,
            end_date,
            from_time: self.from_time,
            to_time,
            quantity_scrap,
            quantity_counted: self.plates_counted,
        };

        // no pending requests, so submit new request
        let request = Request::Finalize(request_data);
        self.submit_request(now, request);
        Ok(false)
    }

    fn poll_response(&mut self) -> anyhow::Result<Response> {
        let response = self
            .client
            .poll_response()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        Ok(response)
    }

    fn submit_request(&mut self, now: Instant, request: Request)
    {
        if self.last_request_ts + self.request_timeout < now {
            // timeout nor reached, can'T send request yet
            return;
        }

        self.client.queue_request(request).expect("Should be able to enqueue");
        self.last_request_ts = now;
    }
}

#[derive(Debug)]
pub enum PlateSubmitResult {
    InBounds,
    OutOufBOunds,
    NotCounting,
}