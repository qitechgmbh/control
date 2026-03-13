use control_core::converters::angle_converter::{AngleConverter, AngleConverterUom};
use units::{Length, Angle, angle::radian};
use units::length::centimeter;

use crate::types::{Bounds, Point2D};

#[derive(Debug, Clone)]
pub struct FilamentTensionCalculator {
    config: Config,
    distance_limits: Bounds<Length>,
}

#[derive(Debug, Clone)]
pub struct Config {
    tension_arm_angle_bounds: Bounds<Angle>,
    tension_arm_wheel_diameter: Length,
    tension_arm_length: Length,
    tension_arm_origin: Point2D<Length>,
    puller_offset: Point2D<Length>,
    traverse_offset: Point2D<Length>,
}

// constants
impl FilamentTensionCalculator {
    const ANGLE_CONVERTER: AngleConverterUom = AngleConverterUom::new(AngleConverter::y_down_cw());
}

// public interface
impl FilamentTensionCalculator {
    pub fn new(config: Config) -> Self {
        let distance_limits = Self::compute_distance_bounds(&config);

        Self {
            config,
            distance_limits,
        }
    }

    pub fn compute(&self, tension_arm_angle: Angle) -> f64 {
        let distance_current = Self::compute_filament_length(&self.config, tension_arm_angle);

        // convert bounds to f64 so we can call normalize
        let bounds: Bounds<f64> = Bounds {
            min: self.distance_limits.min.get::<centimeter>(),
            max: self.distance_limits.max.get::<centimeter>(),
        };

        let ratio = bounds.normalize(distance_current.get::<centimeter>());
        1.0 - ratio
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

// utils
impl FilamentTensionCalculator {
    fn compute_distance_bounds(config: &Config) -> Bounds<Length> {
        let angle_min = config.tension_arm_angle_bounds.min;
        let angle_max = config.tension_arm_angle_bounds.max;
        let min = Self::compute_filament_length(&config, angle_min);
        let max = Self::compute_filament_length(&config, angle_max);
        Bounds::new(min, max)
    }

    fn compute_filament_length(config: &Config, tension_arm_angle: Angle) -> Length {
        let tension_arm_tip = Self::compute_tension_arm_tip(config, tension_arm_angle);

        let puller_offset = Self::point_length_to_point_f64(config.puller_offset);
        let traverse_offset = Self::point_length_to_point_f64(config.traverse_offset);

        // compute total length
        let puller_to_tip = puller_offset.distance_to(tension_arm_tip);
        let tip_to_traverse = tension_arm_tip.distance_to(traverse_offset);
        let total_length = puller_to_tip + tip_to_traverse;

        Length::new::<centimeter>(total_length)
    }

    fn compute_tension_arm_tip(config: &Config, angle: Angle) -> Point2D<f64> {
        let angle_rad = Self::ANGLE_CONVERTER.decode(angle).get::<radian>();

        let wheel_diameter = config.tension_arm_wheel_diameter.get::<centimeter>();

        let length = config.tension_arm_length.get::<centimeter>();
        let origin = Self::point_length_to_point_f64(config.tension_arm_origin);

        let x = length.mul_add(angle_rad.sin(), origin.x);
        let y = length.mul_add(angle_rad.cos(), origin.y);

        // translate the tip down to account for the wheel diameter
        let y = y + wheel_diameter;

        Point2D::new(x, y)
    }

    fn point_length_to_point_f64(point: Point2D<Length>) -> Point2D<f64> {
        let x = point.x.get::<centimeter>();
        let y = point.y.get::<centimeter>();
        Point2D::new(x, y)
    }

    fn normalize_length(bounds: Bounds<Length>, value: Length) -> f64 {
        let bounds: Bounds<f64> = Bounds {
            min: bounds.min.get::<centimeter>(),
            max: bounds.min.get::<centimeter>(),
        };

        bounds.normalize(value.get::<centimeter>())
    }
}
