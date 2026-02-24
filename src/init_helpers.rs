use macroquad::color::BLUE;
use macroquad::math::DVec2;
use macroquad::rand::gen_range;
use crate::constants::*;
use crate::helpers::Particle;
use std::f64::consts::TAU;
pub fn initialize_bodies_spiro(num_bodies_added: usize, system: &mut [Particle; NUMBER_OF_BODIES]) {
    let comet_orbital_radius: f64 = gen_range(
        COMET_ORBITAL_RADIUS_VARIANCE_MIN,
        COMET_ORBITAL_RADIUS_VARIANCE_MAX,
    ) * COMET_ORBITAL_RADIUS;
    for i in 0..EARTH_NUMBER {
        let angular_position: f64 = TAU * i as f64 / EARTH_NUMBER as f64;
        let earth_x_position: f64 =
            CENTER_COORDS[0] + angular_position.cos() * EARTH_ORBITAL_RADIUS;
        let earth_y_position: f64 =
            CENTER_COORDS[1] + angular_position.sin() * EARTH_ORBITAL_RADIUS;
        let earth_position: DVec2 = DVec2::new(earth_x_position, earth_y_position);

        let velocity_direction: f64 = angular_position + 0.5 * std::f64::consts::PI;
        let orbital_speed = 0.3 * EARTH_ORBITAL_VELOCITY;
        let earth_x_velocity: f64 = velocity_direction.cos() * orbital_speed;
        let earth_y_velocity: f64 = velocity_direction.sin() * orbital_speed;

        let earth_velocity: DVec2 = DVec2::new(earth_x_velocity, earth_y_velocity);

        let mut new_planet: Particle = Particle {
            mass: EARTH_MASS,
            position: earth_position,
            velocity: earth_velocity,
            radius: EARTH_RADIUS,
            color: BLUE,
            name: String::from(format!("Planet {}", i + 1)),
            kinetic_energy: 0.,
        };
        new_planet.update_kinetic_energy();
        system[i + 1] = new_planet;
        if system[i + 1].mass >= 0.2 * EARTH_MASS {
            num_important_bodies += 1;
        };
    }
}
