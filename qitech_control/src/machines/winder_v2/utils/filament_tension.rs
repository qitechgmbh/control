use euclid::Point2D;
use qitech_control_core::converters::AngleConverter;
use qitech_control_core::converters::AngleConverterUom;
use qitech_lib::units::ConstZero;
use qitech_lib::units::angle::radian;
use qitech_lib::units::f64::*;
use qitech_lib::units::length::centimeter;
use qitech_lib::units::ratio::ratio;

// The "tension" of the filament is not linear regarding the angle of the tension arm since it moves in an angular motion.
// With this calculator we can calculate the filament length and tension based on the angle of the tension arm using geometry.
#[derive(Debug, Clone)]
pub struct FilamentTensionCalculator {
    point_puller: Point2D<f64, ()>,
    tension_arm_origin: Point2D<f64, ()>,
    traverse_point: Point2D<f64, ()>,
    // In mm
    arm_length: f64,
    /// In Y-Flipped CW rotation system
    min_angle: Angle,
    /// In Y-Flipped CW rotation system
    max_angle: Angle,
    min_distance: Length,
    max_distance: Length,
    angle_converter: AngleConverterUom,
}

impl FilamentTensionCalculator {
    /// [`max_angle_deg`] in Y-Flipped CW roation system
    pub fn new(min_angle: Angle, max_angle: Angle) -> Self {
        let mut calculator = Self {
            point_puller: Point2D::<f64, ()>::new(7.0, -10.0),
            tension_arm_origin: Point2D::<f64, ()>::new(0.0, 0.0),
            traverse_point: Point2D::<f64, ()>::new(-42.0, 15.0),
            arm_length: 18.0,
            min_angle,
            max_angle,
            min_distance: Length::ZERO, // Will be calculated
            max_distance: Length::ZERO, // Will be calculated
            angle_converter: AngleConverterUom::new(AngleConverter::y_down_cw()),
        };

        // Calculate min and max distances
        calculator.min_distance = calculator.calc_filament_length(min_angle);
        calculator.max_distance = calculator.calc_filament_length(max_angle);
        calculator
    }

    // Calculate filament tension as a value between 0.0 (low) and 1.0 (high).
    // `1.0` means the minimum amount of filament is in the tensioning system (high tension).
    // `0.0` means the maximum amount of filament is in the tensioning system (low tension).
    // `tension_arm_angle` uses the Y-flipped CW rotation system.
    pub fn calc_filament_tension(&self, tension_arm_angle: Angle) -> Option<f64> {
        if tension_arm_angle < self.min_angle || tension_arm_angle > self.max_angle {
            // out of bounds
            return None;
        };

        let current_distance = self.calc_filament_length(tension_arm_angle);
        let distance_range = self.max_distance - self.min_distance;

        let normalized = ((current_distance - self.min_distance) / distance_range)
            .get::<ratio>()
            .clamp(0.0, 1.0);

        Some(1.0 - normalized)
    }

    /// Calculate the filament length for a given tension arm angle
    /// - [`tension_arm_angle_deg`] in Y-Flipped CW rotation system
    fn calc_filament_length(&self, tension_arm_angle: Angle) -> Length {
        let tension_arm_angle_rad = self
            .angle_converter
            .decode(tension_arm_angle)
            .get::<radian>();

        // Calculate tension arm tip position (flipped Y-axis: 0° = down, 90° = left)
        let tension_arm_tip = Point2D::<f64, ()>::new(
            self.arm_length
                .mul_add(tension_arm_angle_rad.sin(), self.tension_arm_origin.x),
            self.arm_length
                .mul_add(tension_arm_angle_rad.cos(), self.tension_arm_origin.y),
        );

        // translate the tip 2cm down to account for the wheel diameter
        let tension_arm_tip = Point2D::<f64, ()>::new(
            tension_arm_tip.x,
            tension_arm_tip.y + 2.0, // 2 cm down
        );

        // Calculate total filament distance
        let length_cm = self.point_puller.distance_to(tension_arm_tip)
            + tension_arm_tip.distance_to(self.traverse_point);

        Length::new::<centimeter>(length_cm)
    }
}
