use macroquad::color::*;
use serde::{Deserialize, Serialize};
use crate::helpers::Particle;
use macroquad::math::DVec2;
use ureq::get;


pub fn get_values_from_horizon(body_name: String) -> Particle {


    let horizons_output: String = ureq::get("http://example.com")
        .header("Example-Header", "header value")
        .call()
        .body_mut()
        .read_to_string();




    let body = Particle {
        mass: 0.,
        position: DVec2::new(0., 0.),
        velocity: DVec2::new(0., 0.),
        radius: 0.,
        color: WHITE,
        name: body_name,
    };
    body
}