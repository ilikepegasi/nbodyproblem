use macroquad::color::*;

pub static HORIZONS_IDS: phf::Map<&str, u32> = phf::phf_map! {
    "mercury"   => 199,
    "venus"     => 299,
    "earth"     => 399,
    "mars"      => 499,
    "jupiter"   => 599,
    "saturn"    => 699,
    "uranus"    => 799,
    "neptune"   => 899,
    "luna"      => 301,
    "sun"       => 10,
};
pub static HORIZONS_COLORS: phf::Map<&str, Color> = phf::phf_map! {
    "mercury"   => GRAY,
    "venus"     => Color::from_rgba(248, 226, 176, 200),
    "earth"     => BLUE,
    "mars"      => RED,
    "jupiter"   => ORANGE,
    "saturn"    => YELLOW,
    "uranus"    => SKYBLUE,
    "neptune"   => BLUE,
    "luna"      => LIGHTGRAY,
    "sun"       => YELLOW,
};

pub static MAJOR_BODIES: [&str; 10] = [
    "mercury", "sun", "venus", "earth", "mars", "jupiter", "saturn", "uranus",
    "neptune", "luna",
];

pub static BODY_MASS_KG: phf::Map<&str, f64> = phf::phf_map! {
    "mercury" => 3.301e+23,
    "venus"   => 4.867e+24,
    "earth"   => 5.972e+24,
    "mars"    => 6.417e+23,
    "jupiter" => 1.898e+27,
    "saturn"  => 5.683e+26,
    "uranus"  => 8.681e+25,
    "neptune" => 1.024e+26,
    "luna"    => 7.342e+22,
    "sun"     => 1.989e+30,
};
// Body Radii in meters
pub static BODY_RADIUS_M: phf::Map<&str, f64> = phf::phf_map! {
    "mercury" => 2.439e+06,
    "venus"   => 6.051e+06,
    "earth"   => 6.371e+06,
    "mars"    => 3.389e+06,
    "jupiter" => 6.991e+07,
    "saturn"  => 5.823e+07,
    "uranus"  => 2.536e+07,
    "neptune" => 2.462e+07,
    "luna"    => 1.737e+06,
    "sun"     => 6.957e+08,
};