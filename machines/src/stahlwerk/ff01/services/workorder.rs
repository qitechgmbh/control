use std::time::{Duration, Instant};

use anyhow::anyhow;
use stahlwerk_extension::ff01::{Entry, ProxyClient, Request, Response};

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
        let Some(mut entry) = self.get_entry(now)? else {
            return Ok(());
        };

        if entry.scrap_quantity > 0.0 {
            self.finalize_workorder(now, entry)?;
            return Ok(());
        }

        entry.scrap_quantity = self.fetch_scrap_quantity(now, &entry)?.unwrap_or(0.0);

        // move entry back into self.entry
        self.entry = Some(entry);

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

    fn fetch_scrap_quantity(&mut self, now: Instant, entry: &Entry) -> anyhow::Result<Option<f64>> {
        // awaiting pending request
        if self.client.has_pending_request() {
            if let Response::GetScrapQuantity(scrap_quantity) = self.poll_response()? {
                return Ok(scrap_quantity);
            } 

            return Err(anyhow!("Tag Mismatch"));
        }

        // no pending requests, so submit new request
        let request = Request::GetScrapQuantity(entry);
        self.submit_request(now, request);
        Ok(None)
    }

    fn finalize_workorder(&mut self, now: Instant, entry: Entry) -> anyhow::Result<()> {
        // awaiting pending request
        if self.client.has_pending_request() {
            if let Response::Finalize = self.poll_response()? {
                return Ok(());
            } 

            return Err(anyhow!("Tag Mismatch"));
        }

        // no pending requests, so submit new request
        let request = Request::Finalize(&entry, self.plates_counted);
        self.submit_request(now, request);
        self.entry = Some(entry);
        Ok(())
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