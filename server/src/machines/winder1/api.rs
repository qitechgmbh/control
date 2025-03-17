use super::WinderV1;
use crate::machines::MachineApi;
use serde_json::Value;

impl MachineApi for WinderV1 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn api_subscribe(&self) -> () {
        todo!()
    }
}
