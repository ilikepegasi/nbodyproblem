use macroquad::color::*;
use macroquad::math::DVec2;
use macroquad::rand::gen_range;
use crate::constants::*;
use crate::helpers::{calculate_orbital_speed, Particle};
use std::f64::consts::TAU;
pub enum Variance {
    WithVariance (f64, f64), // (Min Variance, Max Variance)
    NoVariance,
}

pub enum CenterObjectValues {
    CenterObjectExists (f64, DVec2),
    NoCenterObject,
}


pub fn initialize_bodies_spiro(bodies_to_add: &usize,
            num_bodies_added: &usize,
                               orbital_radius: &f64,
                               mass: &f64,
                               color: &Color,
                               radius: &f64,
                               orbital_speed_factor: &f64,
                               system: &mut Vec<Particle>,
                               initial_angular_offset: &f64,
                               orbital_radius_variance: Variance,
                               mass_variance: Variance,
                               center_object_values: &CenterObjectValues,
                               category_name: &str,
) -> (usize, usize) /* This will return the amount of significant bodies added */ {
    let mut orbital_speed: f64;
    let mut orbital_radius_actual = *orbital_radius;
    let mut mass_actual: f64 = *mass;
    let mut num_important_bodies_added= 0;
    let mut bodies_added= 0;


    if let Variance::WithVariance(min_variance, max_variance) = orbital_radius_variance {
        orbital_radius_actual = *orbital_radius * gen_range(min_variance, max_variance);
    }
    if let Variance::WithVariance(min_variance, max_variance) = mass_variance {
        mass_actual = *mass * gen_range(min_variance, max_variance);
    }


    for i in 0..*bodies_to_add {
        let angular_position: f64 = (TAU * i as f64 + initial_angular_offset) / *bodies_to_add as f64;
        let body_x_position: f64 =
            CENTER_COORDS[0] + angular_position.cos() * orbital_radius_actual;
        let body_y_position: f64 =
            CENTER_COORDS[1] + angular_position.sin() * orbital_radius_actual;
        let body_position: DVec2 = DVec2::new(body_x_position, body_y_position);
        if let CenterObjectValues::CenterObjectExists(center_mass, center_position) = center_object_values {
            orbital_speed = orbital_speed_factor * calculate_orbital_speed(&center_mass, &center_position, body_position);
        } else {
            orbital_speed = 0.0;
        }
        let velocity_direction: f64 = angular_position + 0.5 * std::f64::consts::PI;
        let body_x_velocity: f64 = velocity_direction.cos() * orbital_speed;
        let body_y_velocity: f64 = velocity_direction.sin() * orbital_speed;

        let earth_velocity: DVec2 = DVec2::new(body_x_velocity, body_y_velocity);

        let mut new_body: Particle = Particle {
            mass: mass_actual,
            position: body_position,
            velocity: earth_velocity,
            radius: *radius,
            color: *color,
            name: String::from(format!("{} {}", category_name, i + 1)),
            kinetic_energy: 0.,
        };
        new_body.update_kinetic_energy();
        system.push(new_body);
        if system[i + *num_bodies_added].mass >= IMPORTANT_BODY_MASS_MIN {
            num_important_bodies_added += 1;
        };
        bodies_added += 1;
    }
    (bodies_added, num_important_bodies_added)
}
