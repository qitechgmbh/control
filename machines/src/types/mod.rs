mod bounds;
pub use bounds::Bounds;
pub use bounds::ExceededBound;
pub use bounds::BoundedValue;
pub use bounds::ClampResult;

mod direction;
pub use direction::Direction;

pub type Point2D<T> = euclid::Point2D<T, ()>;
pub type Point3D<T> = euclid::Point3D<T, ()>;