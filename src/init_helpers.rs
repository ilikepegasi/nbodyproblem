use std::alloc::System;
use crate::constants::*;
use crate::helpers::{Particle, calculate_orbital_speed};
use macroquad::color::*;
use macroquad::math::DVec2;
use macroquad::rand::gen_range;
use std::f64::consts::TAU;
use std::string::ToString;
use crate::init_helpers::CenterObjectValues::CenterObjectExists;
use crate::init_helpers::ScenarioKey::Scenario;

#[derive(Debug)]
pub enum ScenarioKey {
    Scenario(String, usize),
}


pub enum Variance {
    WithVariance(f64, f64), // (Min Variance, Max Variance)
    NoVariance,
}

pub enum CenterObjectValues {
    CenterObjectExists(f64, DVec2),
    NoCenterObject,
}


pub fn initialize_from_scenario(scenario: usize, system: &mut Vec<Particle>, scenario_list: &Vec<ScenarioKey>) -> (usize, usize, f64) {
    let scenario_name = scenario_list.iter().find_map(|k| match k {
        Scenario(name, key) if *key == scenario => Some(name.as_str()),
        _ => None,
    }).expect("Invalid scenario key");
    let mut total_bodies_added = 0;
    let mut sim_seconds_per_frame = 2e4;

    let mut num_important_bodies = 0;
    match scenario_name {
        "Spirograph" => {
            let mut star: Particle = Particle {
                mass: STAR_MASS,
                position: DVec2::new(CENTER_COORDS[0], CENTER_COORDS[1]),
                velocity: DVec2::new(0., 0.),
                radius: STAR_RADIUS,
                color: YELLOW,
                name: String::from("Sun"),
                kinetic_energy: 0.,
            };
            star.update_kinetic_energy();
            system.push(star);
            num_important_bodies += 1;
            total_bodies_added += 1;
            let center_object_values = CenterObjectExists(system[0].mass, system[0].position);

            let bodies_values_delta = initialize_bodies_spiro(
                &EARTH_NUMBER,
                &total_bodies_added,
                &(EARTH_ORBITAL_RADIUS),
                &EARTH_MASS,
                &WHITE,
                &EARTH_RADIUS,
                &0.5,
                system,
                &DEFAULT_ANGULAR_OFFSET,
                Variance::NoVariance,
                Variance::NoVariance,
                &center_object_values,
                "Planet",
            );
            sim_seconds_per_frame = FIGURE_8_SECONDS_PER_FRAME;

            total_bodies_added += bodies_values_delta.0;
            num_important_bodies += bodies_values_delta.1;
            println!("Spirograph scenario initialized with key {}", scenario);

        }
        "Figure 8" => {
            let bodies_values_delta = initialize_figure_8_scenario(system, &EARTH_ORBITAL_RADIUS, &EARTH_MASS, &EARTH_RADIUS);
            total_bodies_added += bodies_values_delta.0;
            num_important_bodies += bodies_values_delta.1;
            println!("Figure 8 initialized with scale {} AU", &EARTH_ORBITAL_RADIUS / &EARTH_ORBITAL_RADIUS);
            for body in system {
                println!("{}", body.position/EARTH_ORBITAL_RADIUS);

            }
            sim_seconds_per_frame = SPIRO_SECONDS_PER_FRAME;

        }
        _ => {unreachable!("Initialization failed")}
    }
    (total_bodies_added, num_important_bodies, sim_seconds_per_frame)
}


pub fn initialize_bodies_spiro(
    bodies_to_add: &usize,
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
    let mut num_important_bodies_added = 0;
    let mut bodies_added = 0;

    if let Variance::WithVariance(min_variance, max_variance) = orbital_radius_variance {
        orbital_radius_actual = *orbital_radius * gen_range(min_variance, max_variance);
    }
    if let Variance::WithVariance(min_variance, max_variance) = mass_variance {
        mass_actual = *mass * gen_range(min_variance, max_variance);
    }

    for i in 0..*bodies_to_add {
        let angular_position: f64 =
            (TAU * i as f64 + initial_angular_offset) / *bodies_to_add as f64;
        let body_x_position: f64 =
            CENTER_COORDS[0] + angular_position.cos() * orbital_radius_actual;
        let body_y_position: f64 =
            CENTER_COORDS[1] + angular_position.sin() * orbital_radius_actual;
        let body_position: DVec2 = DVec2::new(body_x_position, body_y_position);
        if let CenterObjectExists(center_mass, center_position) =
            center_object_values
        {
            orbital_speed = orbital_speed_factor
                * calculate_orbital_speed(&center_mass, &center_position, body_position);
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

pub fn initialize_figure_8_scenario(system: &mut Vec<Particle>, length_scale: &f64, body_masses: &f64, body_radii: &f64) -> (usize, usize) {
    let canonical_figure_8_positions = [
        DVec2::new(-0.97000436,  0.24308753),
        DVec2::new( 0.97000436, -0.24308753),
        DVec2::new( 0.0,         0.0),
        // Unitless
    ];
    let canonical_figure_8_velocities = [
        DVec2::new(-0.46620368, -0.43236573),
        DVec2::new(-0.46620368, -0.43236573),
        DVec2::new( 0.93240737,  0.86473146),
        // Unitless
    ];
    let mut real_positions: [DVec2;3] = [DVec2::ZERO; 3];
    for i in 0..3 {
        real_positions[i] = *length_scale * canonical_figure_8_positions[i];
    }
    let mut real_velocities: [DVec2;3] = [DVec2::ZERO; 3];
    for i in 0..3 {
        real_velocities[i] = canonical_figure_8_velocities[i] * (G * body_masses / *length_scale).powf(0.5);
    }
    let center = DVec2::new(CENTER_COORDS[0], CENTER_COORDS[1]);
    for i in 0..3 {
        let mut new_body = Particle {
            mass: *body_masses,
            position: real_positions[i] + center,
            velocity: real_velocities[i],
            radius: *body_radii,
            color: [RED, BLUE, GREEN][i],
            name: format!("Figure 8 Body {}", i),
            kinetic_energy: 0.0,
        };
        new_body.update_kinetic_energy();
        system.push(new_body);
    }

    (3,3)
}


