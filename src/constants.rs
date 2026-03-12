use macroquad::math::DVec2;

// Masses in kilograms

pub const STAR_MASS: f64 = 1.9891e30;
pub const EARTH_MASS: f64 = 5.9722e24;

pub const EARTH_RADIUS: f64 = 6.3781e6;
pub const STAR_RADIUS: f64 = 6.957e8;
pub const COMET_RADIUS: f64 = 7.4e3;

// Orbital Radii
pub const EARTH_ORBITAL_RADIUS: f64 = 1.496e11;
pub const AU: f64 = 149597870700.;

// Orbital Velocities in meters / second
// pub const EARTH_ORBITAL_VELOCITY: f64 = 2.978e4;

// Fundamental Constants
pub const G: f64 = 6.674e-11; //meters^3 kilograms^−1 seconds^−2
pub const SECONDS_IN_YEAR: f64 = 31556926.; // seconds/year

// Simulation Parameters
// pub const VIEWER_SECONDS_PER_FRAME: f64 = 1./ FRAMES_PER_VIEWER_SECOND as f64;
// pub const FRAMES_PER_VIEWER_SECOND: usize = 80;

pub const COLLISION_MIN_MASS: f64 = 1.0;

// Simulation Initialization Parameters
//pub const COMET_MASS_VARIANCE_MAX: f64 = 0.8;
//pub const COMET_ORBITAL_RADIUS_VARIANCE_MAX: f64 = 1.2;
//pub const COMET_MASS_VARIANCE_MIN: f64 = 0.8;
//pub const COMET_ORBITAL_RADIUS_VARIANCE_MIN: f64 = 0.01;
pub const YEARS_PER_FRAME_SOLAR_SYS: f64 = 0.0005;
pub const DEFAULT_ANGULAR_OFFSET: f64 = 0.;
pub const FIGURE_8_SECONDS_PER_FRAME: f64 = 8e6;
pub const SPIRO_SECONDS_PER_FRAME: f64 = 2e4;
pub const SOLAR_SYS_SECONDS_PER_FRAME: f64 = YEARS_PER_FRAME_SOLAR_SYS * SECONDS_IN_YEAR;

pub const TICKS_PER_FRAME_FIG8: usize = 120; //divide by zero error if 1 IDK why

pub const TICKS_PER_FRAME_SPIRO: usize = 20; //divide by zero error if 1 IDK why

pub const TICKS_PER_FRAME_SOLAR_SYSTEM: usize = 300; //divide by zero error if 1 IDK why
pub const EARTH_NUMBER_MAX: usize = 600;
pub const EPSILON: f64 = COMET_RADIUS;
pub const COLLIDED_POSITION: DVec2 =
    DVec2::new(EARTH_ORBITAL_RADIUS * 1e8, EARTH_ORBITAL_RADIUS * 1e8);

// Data Parameters
pub const ROW_LIMIT: usize = 24000;
pub const PHYSICAL_DATA_INTERVAL: usize = 1;
pub const YEARS_OF_WRITING_SPIRO: f32 = 8.0;
pub const YEARS_OF_WRITING_SOLAR_SYSTEM: f32 = 24.0;
pub const YEARS_OF_WRITING_FIG8: f32 = 1000.0;

pub const LEFT_PAD: usize = 6;
pub const COLUMNS_PER_OBJECT: usize = 2;

// Graphics Parameters
pub const SCREEN_SIZE_PIXELS: u32 = 1000;

pub const OFFSET_VELOCITY: f32 = 6.0;
pub const ZOOM_VELOCITY: f64 = 0.95;
pub const SCREEN_SIZE_SPIRO_METERS: f64 = 2.5 * AU;
pub const SCREEN_SIZE_FIG8_METERS: f64 = 2.5 * AU;
pub const SCREEN_SIZE_SOLAR_SYS_METERS: f64 = 65.0 * AU;
pub const OLD_FRAME_LIMIT_SPIRO: usize = 2usize.pow(9);
pub const OLD_FRAME_LIMIT_FIG8: usize = 2usize.pow(11);
pub const OLD_FRAME_LIMIT_SOLAR_SYS: usize = 2usize.pow(11);

pub const SMALL_RADIUS: f64 = EARTH_RADIUS / 10.;
pub const MAX_TRAIL_LINE_LEN: f32 = EARTH_ORBITAL_RADIUS as f32;
pub const MIN_RADIUS_MAX_COLOR: f64 = 0.2 * EARTH_ORBITAL_RADIUS;
pub const MAX_RADIUS_MIN_COLOR: f64 = 1.0 * EARTH_ORBITAL_RADIUS;
pub const MAX_VIOLET_HUE: f32 = 0.72;
//pub const MIN_RED_HUE: f32 = 0.0;

pub const IMPORTANT_BODY_MASS_MIN: f64 = 0.2 * EARTH_MASS;

// Graphical Sizes
pub const TRAIL_RADIUS: f32 = 1.;
pub const MAX_RADIUS_PIXELS: f32 = 4.0;
pub const MIN_RADIUS_PIXELS: f32 = 1.0;
