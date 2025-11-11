use control_core::converters::angle_converter::{AngleConverter, AngleConverterUom};
use euclid::Point2D;
use units::ConstZero;
use units::angle::radian;
use units::f64::*;
use units::length::centimeter;
use units::ratio::ratio;

/// The "tension" of the filament is not linear regarding the angle of the tension arm since it moves in an angular motion.
///
/// With this calculator we can calculate the filament length and tension based on the angle of the tension arm using geometry.
///
/// ⠉⠢⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇ 0.0
/// ⠀⠀⠀⠉⠢⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
/// ⠀⠀⠀⠀⠀⠀⠑⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
/// ⠀⠀⠀⠀⠀⠀⠀⠀⠈⠒⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
/// ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠒⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
/// ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
/// ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
/// ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
/// ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
/// ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
/// ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
/// ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠤⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
/// ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠒⠤⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
/// ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠑⠒⠤⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
/// ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠒⠒⠤⠤⢄⣀⣀⡀⠀⠀⠀⠀⠀⠀⡇
/// ⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠈⠉⠉⠉⠉⠉⠉⠁ 1.0
/// 10.0                                                    90.0
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
    pub angle_converter: AngleConverterUom,
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

    /// Calculate the filament length for a given tension arm angle
    ///
    /// - [`tension_arm_angle_deg`] in Y-Flipped CW rotation system
    ///
    /// Returns the filament length in mm.
    pub fn calc_filament_length(&self, tension_arm_angle: Angle) -> Length {
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

    /// Calculate the filament buffer as a value between 0.0 (min) and 1.0 (max)
    ///
    /// `1.0` means the minimum amount of filament is in the tensioning system (high tension)
    ///
    /// `0.0` means the maximum amount of filament is in the tensioning system (low tension)
    ///
    /// - [`tension_arm_angle`] in Y-Flipped CW rotation system
    pub fn calc_filament_tension(&self, tension_arm_angle: Angle) -> f64 {
        let current_distance = self.calc_filament_length(tension_arm_angle);

        let distance_range = self.max_distance - self.min_distance;

        1.0 - ((current_distance - self.min_distance) / distance_range)
            .get::<ratio>()
            .clamp(0.0, 1.0)
    }

    /// Get the optimal angle (minimum filament length)
    ///
    /// Returns the angle in Y-Flipped CW rotation system.
    pub fn get_min_angle(&self) -> Angle {
        self.min_angle
    }

    /// Get the maximum reference angle
    ///
    /// Returns the angle in Y-Flipped CW rotation system.
    pub fn get_max_angle(&self) -> Angle {
        self.max_angle
    }

    /// Get the minimum filament distance
    ///
    pub fn get_min_distance(&self) -> Length {
        self.min_distance
    }

    /// Get the maximum filament distance
    ///
    pub fn get_max_distance(&self) -> Length {
        self.max_distance
    }
}

#[cfg(test)]
mod tests {
    use textplots::{Chart, Plot, Shape};

    use units::angle::degree;

    use super::*;

    #[test]
    fn test_plot_filament_distance() {
        let min_angle = 90.0;
        let max_angle = 10.0;

        let calculator = FilamentTensionCalculator::new(
            Angle::new::<degree>(min_angle),
            Angle::new::<degree>(max_angle),
        );

        // Generate data points from 0 to 90 degrees
        let data: Vec<(f32, f32)> = (0..=360)
            .map(|angle_deg| {
                let distance =
                    calculator.calc_filament_length(Angle::new::<degree>(angle_deg as f64));
                (angle_deg as f32, distance.get::<centimeter>() as f32)
            })
            .collect();

        // Create and display the plot
        Chart::new(120, 60, max_angle as f32, min_angle as f32)
            .lineplot(&Shape::Lines(&data))
            .display();

        println!("Filament distance vs tension arm angle (0-90 degrees)");
        println!(
            "Min distance: {:.2}",
            calculator.get_min_distance().get::<centimeter>()
        );
        println!(
            "Max distance: {:.2}",
            calculator.get_max_distance().get::<centimeter>()
        );
    }

    #[test]
    fn test_filament_tension() {
        let min_angle = 90.0;
        let max_angle = 10.0;
        let calculator = FilamentTensionCalculator::new(
            Angle::new::<degree>(min_angle),
            Angle::new::<degree>(max_angle),
        );

        // Generate data points from 0 to 90 degrees
        let data: Vec<(f32, f32)> = (0..=360)
            .map(|angle_deg| {
                let tension =
                    calculator.calc_filament_tension(Angle::new::<degree>(angle_deg as f64));
                (angle_deg as f32, tension as f32)
            })
            .collect();

        // Create and display the plot
        Chart::new(120, 60, max_angle as f32, min_angle as f32)
            .lineplot(&Shape::Lines(&data))
            .display();

        println!("Filament tension vs tension arm angle (0-90 degrees)");
        println!(
            "Tension at min angle ({}°): {:.2}",
            min_angle,
            calculator.calc_filament_tension(calculator.get_min_angle())
        );
        println!(
            "Tension at max angle ({}°): {:.2}",
            max_angle,
            calculator.calc_filament_tension(calculator.get_max_angle())
        );
    }
}
