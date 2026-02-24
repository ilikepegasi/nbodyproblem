use macroquad::color::BLUE;
use macroquad::math::DVec2;
use macroquad::rand::gen_range;
use crate::constants::*;
use crate::helpers::Particle;
use std::f64::consts::TAU;
pub enum Variance {
    With_Variance (f64, f64), // (Min Variance, Max Variance)
    No_Variance,
}


pub fn initialize_bodies_spiro(num_bodies_added: usize,
                               orbital_radius: f64,
                               mass: f64,
                               system: &mut [Particle; NUMBER_OF_BODIES],
                               radius_variance: Variance,
                               mass_variance: Variance,
) -> usize /* This will return the amount of significant bodies added */ {

    if let Variance::With_Variance(min_variance, max_variance) = radius_variance {
        let orbital_radius = orbital_radius * gen_range(min_variance, max_variance);
    }
    if let Variance::With_Variance(min_variance, max_variance) = mass_variance {
        let mass = mass * gen_range(min_variance, max_variance);
    }

    for i in 0..EARTH_NUMBER {
        let angular_position: f64 = TAU * i as f64 / num_bodies_added as f64;
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
