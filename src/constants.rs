



// Masses in kilograms
pub const STAR_MASS: f64 = 1.9891e30;
pub const EARTH_MASS: f64 = 5.9722e24 ;
pub const COMET_MASS: f64 = 2.2e14;
pub const JUPITER_MASS: f64 = 1.898e27;


// Body Radii in meters
pub const PLANET_RADIUS: f64 = 6.3781e6;
pub const STAR_RADIUS: f64 = 6.957e8;
pub const COMET_RADIUS: f64 = 7.4e3;


// Orbital Radii
pub const EARTH_ORBITAL_RADIUS: f64 = 1.496e11;
pub static COMET_ORBITAL_RADIUS: f64 = EARTH_ORBITAL_RADIUS * 0.75;
 
// Orbital Velocity in meters / second
pub const EARTH_ORBITAL_VELOCITY: f64 = 2.978e4;
pub static COMET_ORBITAL_VELOCITY: f64 = 3.44e4;

// Fundamental Constants
pub const G: f64 = 6.674e-11; //meters^3 kilograms^−1 seconds^−2
pub const SECONDS_IN_YEAR: f64 = 31556926.; // seconds/year



// Simulation Parameters
pub const NUMBER_OF_BODIES: usize = 13usize.pow(4);
pub const FRAMERATE: f64 = 1./60.;
pub const DT: f64 = 1e6 * FRAMERATE;
pub const COMET_VARIANCE_MAX: f64 = 1.2;
pub const COMET_VARIANCE_MIN: f64 = 0.8;


// Data Parameters
pub const ROW_LIMIT: usize = 1800;


// Graphics Parameters
 pub const VISIBILITY: f32 = 1e9;
pub const SCREEN_SIZE: i32 = 800;
pub const SCALING_FACTOR: f64 = 2.5;
pub const OLD_FRAME_LIMIT: usize = (2usize).pow(8);
pub const NUM_OLD_POSITION_LIMIT: usize = OLD_FRAME_LIMIT * NUMBER_OF_BODIES;

// Graphical Sizes
pub const STAR_VISIBLE_RADIUS: f32 = 5.; // In pixels
pub const PLANET_VISIBLE_RADIUS: f32 = 3.; // In pixels
pub const COMET_VISIBLE_RADIUS: f32 = 0.6; // In pixels
pub const TRAIL_RADIUS: f32 = 0.5;

