use opentelemetry::{global, metrics::Gauge};

pub struct Metrics {
    pub diameter: Gauge<f64>,
    pub x_diameter: Gauge<f64>,
    pub y_diameter: Gauge<f64>,
    pub roundness: Gauge<f64>,
}
 
impl Metrics {   
    pub fn new() ->Self {
        println!("Yes indeed!");

        let meter = global::meter("winder_v1");

        let diameter = meter
            .f64_gauge("diameter")
            .with_unit("mm")
            .build();

        let x_diameter = meter
            .f64_gauge("x_diameter")
            .with_unit("mm")
            .build();

        let y_diameter = meter
            .f64_gauge("y_diameter")
            .with_unit("mm")
            .build();

        let roundness = meter
            .f64_gauge("roundness")
            .with_unit("%")
            .build();

        Self {
            diameter,
            x_diameter,
            y_diameter,
            roundness,
        }
    }
}