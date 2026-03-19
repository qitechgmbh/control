#[derive(Debug, Clone)]
pub struct PlateDetectTask {
    weight_prev: Option<f64>,
}

impl PlateDetectTask 
{
    pub fn new() -> Self {
        Self { weight_prev: None }
    }

    pub fn check(&mut self, weight: f64) -> bool {
        let Some(weight_prev) = self.weight_prev else {
            self.weight_prev = Some(weight);
            return false;
        };

        let reached_zero = weight_prev > 0.0 && weight == 0.0;

        reached_zero
    }

    pub fn reset(&mut self) {
        self.weight_prev = None;
    }
}