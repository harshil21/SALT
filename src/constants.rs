//! Constants used throughout the program.

/// Velocity above which the rocket is considered to be in powered flight:
pub const TAKEOFF_VELOCITY_METERS_PER_SECOND: f32 = 10.0;
/// Velocity % below which the rocket is considered to be in coast:
pub const MAX_VELOCITY_THRESHOLD: f32 = 0.96;

/// Altitude % below which the rocket is considered to be in free fall:
pub const MAX_ALTITUDE_THRESHOLD: f32 = 0.94;

/// Altitude below which the rocket is considered to have landed from free fall:
pub const GROUND_ALTITUDE_METERS: f32 = 15.0;

/// Seconds after which the rocket is considered to have landed:
pub const SECONDS_TO_CONSIDERED_LANDED: u64 = 10;

/// Maximum time we can be in free fall:
pub const MAX_FREE_FALL_SECONDS: u64 = 300;

pub const VELOCITY_FROM_ALTITUDE_WINDOW_SIZE: usize = 15;
pub const ALTITUDE_DEADBAND_METERS: f32 = 0.05;
