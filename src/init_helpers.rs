use crate::constants::*;
use crate::helpers::{Particle, calculate_orbital_speed, get_number_from_user, take_user_choice};
use crate::horizon::get_horizons_data;
use crate::horizons_table::*;
use crate::init_helpers::CenterObjectValues::CenterObjectExists;
use crate::render::ScreenValues;
use macroquad::color::*;
use macroquad::math::{DVec2, Vec2};
use macroquad::rand::gen_range;
use std::f64::consts::TAU;
use std::string::ToString;

#[derive(Debug)]
pub struct ScenarioKey(pub String, pub usize);

pub enum Variance {
    WithVariance(f64, f64), // (Min Variance, Max Variance)
    NoVariance,
}

pub enum CenterObjectValues {
    CenterObjectExists(f64, DVec2),
}

pub struct ConfigValues {
    pub scenario_name: String,
    pub important_bodies_added: usize,
    pub ticks_per_frame: usize,
    pub dt: f64,
    pub color_vel_range: (f64, f64),
    pub trail_length: usize,
    pub years_of_writing: f32,
}

pub fn initialize_from_scenario(
    scenario: usize,
    system: &mut Vec<Particle>,
    scenario_list: &Vec<ScenarioKey>,
    screen_values: &mut ScreenValues,
) -> ConfigValues {
    let scenario_name = scenario_list
        .iter()
        .find_map(|k| match k {
            ScenarioKey(name, key) if *key == scenario => Some(name.as_str()),
            _ => None,
        })
        .expect("Invalid scenario key");
    let mut total_bodies_added = 0;
    let mut sim_seconds_per_frame = 2e4;
    let mut ticks_per_frame = 1;
    let mut important_bodies_added = 0;
    let mut minimum_speed_color: f64 = 0.;
    let mut maximum_speed_color: f64 = 1.;
    let mut years_of_writing = 0.;
    let mut trail_length: usize = 0;
    let mut screen_size_meters: u32 = 0;
    let mut offset_pixels = Vec2::ZERO;

    match scenario_name {
        "Spirograph" => {
            screen_values.initialize(SCREEN_SIZE_PIXELS, SCREEN_SIZE_SPIRO_METERS);

            let star: Particle = Particle {
                mass: STAR_MASS,
                position: DVec2::new(0., 0.),
                velocity: DVec2::new(0., 0.),
                radius: STAR_RADIUS,
                color: YELLOW,
                name: String::from("Sun"),
            };
            system.push(star);
            important_bodies_added += 1;
            total_bodies_added += 1;
            let center_object_values = CenterObjectExists(system[0].mass, system[0].position);
            let mut earth_number: usize;
            loop {
                earth_number =
                    get_number_from_user(&format!("How many Earths? (max {})", EARTH_NUMBER_MAX))
                        as usize;
                if earth_number < EARTH_NUMBER_MAX {
                    break;
                } else {
                    println!("Invalid Earth number: {}", earth_number);
                }
            }
            let bodies_values_delta = initialize_bodies_spiro(
                &earth_number,
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
            sim_seconds_per_frame = SPIRO_SECONDS_PER_FRAME;
            years_of_writing = YEARS_OF_WRITING_SPIRO;

            total_bodies_added += bodies_values_delta.0;
            important_bodies_added += bodies_values_delta.1;
            println!("Spirograph scenario initialized with key {}", scenario);
            ticks_per_frame = TICKS_PER_FRAME_SPIRO;
            trail_length = OLD_FRAME_LIMIT_SPIRO;
            minimum_speed_color = calculate_orbital_speed(
                &system[0].mass,
                &system[0].position,
                DVec2::new(
                    system[0].position.x,
                    system[0].position.y + MAX_RADIUS_MIN_COLOR,
                ),
            )
            .log10();
            maximum_speed_color = calculate_orbital_speed(
                &system[0].mass,
                &system[0].position,
                DVec2::new(
                    system[0].position.x,
                    system[0].position.y + MIN_RADIUS_MAX_COLOR,
                ),
            )
            .log10();
        }
        "Figure 8" => {
            let bodies_values_delta = initialize_figure_8_scenario(
                system,
                &EARTH_ORBITAL_RADIUS,
                &EARTH_MASS,
                &EARTH_RADIUS,
            );
            total_bodies_added += bodies_values_delta.0;
            important_bodies_added += bodies_values_delta.1;
            println!(
                "Figure 8 initialized with scale {} AU",
                &EARTH_ORBITAL_RADIUS / &EARTH_ORBITAL_RADIUS
            );
            for body in system.iter() {
                println!("{}", body.position / EARTH_ORBITAL_RADIUS);
            }

            trail_length = OLD_FRAME_LIMIT_FIG8;
            years_of_writing = YEARS_OF_WRITING_FIG8;
            sim_seconds_per_frame = FIGURE_8_SECONDS_PER_FRAME;
            ticks_per_frame = TICKS_PER_FRAME_FIG8;
            minimum_speed_color = system[0].velocity.length().log10();
            maximum_speed_color = system[2].velocity.length().log10();
            screen_values.initialize(SCREEN_SIZE_PIXELS, SCREEN_SIZE_FIG8_METERS);
        }
        "Solar System" => {
            let bodies_values_delta = initialize_solar_system(system);
            total_bodies_added += bodies_values_delta.0;
            important_bodies_added += bodies_values_delta.1;
            years_of_writing = YEARS_OF_WRITING_SOLAR_SYSTEM;
            ticks_per_frame = TICKS_PER_FRAME_SOLAR_SYSTEM;
            sim_seconds_per_frame = SOLAR_SYS_SECONDS_PER_FRAME;
            trail_length = OLD_FRAME_LIMIT_SOLAR_SYS;

            minimum_speed_color = system[8].velocity.length().log10();
            maximum_speed_color = system[1].velocity.length().log10();
            screen_values.initialize(SCREEN_SIZE_PIXELS, SCREEN_SIZE_SOLAR_SYS_METERS);
        }

        _ => {
            unreachable!("Initialization failed")
        }
    }

    let dt = sim_seconds_per_frame / ticks_per_frame as f64;
    let config_values = ConfigValues {
        scenario_name: scenario_name.to_string(),
        important_bodies_added,
        ticks_per_frame,
        dt,
        color_vel_range: (minimum_speed_color, maximum_speed_color),
        trail_length,
        years_of_writing,
    };

    config_values
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

    for i in 0..*bodies_to_add {
        if let Variance::WithVariance(min_variance, max_variance) = orbital_radius_variance {
            orbital_radius_actual = *orbital_radius * gen_range(min_variance, max_variance);
        }
        if let Variance::WithVariance(min_variance, max_variance) = mass_variance {
            mass_actual = *mass * gen_range(min_variance, max_variance);
        }
        let angular_position: f64 =
            (TAU * i as f64 + initial_angular_offset) / *bodies_to_add as f64;
        let body_x_position: f64 = angular_position.cos() * orbital_radius_actual;
        let body_y_position: f64 = angular_position.sin() * orbital_radius_actual;
        let body_position: DVec2 = DVec2::new(body_x_position, body_y_position);
        if let CenterObjectExists(center_mass, center_position) = center_object_values {
            orbital_speed = orbital_speed_factor
                * calculate_orbital_speed(&center_mass, &center_position, body_position);
        } else {
            orbital_speed = 0.0;
        }
        let velocity_direction: f64 = angular_position + 0.5 * std::f64::consts::PI;
        let body_x_velocity: f64 = velocity_direction.cos() * orbital_speed;
        let body_y_velocity: f64 = velocity_direction.sin() * orbital_speed;

        let earth_velocity: DVec2 = DVec2::new(body_x_velocity, body_y_velocity);

        let new_body: Particle = Particle {
            mass: mass_actual,
            position: body_position,
            velocity: earth_velocity,
            radius: *radius,
            color: *color,
            name: String::from(format!("{} {}", category_name, i + 1)),
        };
        system.push(new_body);
        if system[i + *num_bodies_added].mass >= IMPORTANT_BODY_MASS_MIN {
            num_important_bodies_added += 1;
        };
        bodies_added += 1;
    }
    (bodies_added, num_important_bodies_added)
}

pub fn initialize_figure_8_scenario(
    system: &mut Vec<Particle>,
    length_scale: &f64,
    body_masses: &f64,
    body_radii: &f64,
) -> (usize, usize) {
    let canonical_figure_8_positions = [
        DVec2::new(-0.97000436, 0.24308753),
        DVec2::new(0.97000436, -0.24308753),
        DVec2::new(0.0, 0.0),
        // Unitless
    ];
    let canonical_figure_8_velocities = [
        DVec2::new(-0.46620368, -0.43236573),
        DVec2::new(-0.46620368, -0.43236573),
        DVec2::new(0.93240737, 0.86473146),
        // Unitless
    ];
    let mut real_positions: [DVec2; 3] = [DVec2::ZERO; 3];
    for i in 0..3 {
        real_positions[i] = *length_scale * canonical_figure_8_positions[i];
    }
    let mut real_velocities: [DVec2; 3] = [DVec2::ZERO; 3];
    for i in 0..3 {
        real_velocities[i] =
            canonical_figure_8_velocities[i] * (G * body_masses / *length_scale).powf(0.5);
    }
    for i in 0..3 {
        let new_body = Particle {
            mass: *body_masses,
            position: real_positions[i],
            velocity: real_velocities[i],
            radius: *body_radii,
            color: [RED, BLUE, GREEN][i],
            name: format!("Figure 8 Body {}", i),
        };
        system.push(new_body);
    }

    (3, 3)
}

pub fn initialize_solar_system(system: &mut Vec<Particle>) -> (usize, usize) {
    let horizons_values = get_horizons_data();
    for value in horizons_values.iter() {
        let new_body = Particle {
            mass: BODY_MASS_KG[&value.name],
            radius: BODY_RADIUS_M[&value.name],
            position: DVec2::new(km_to_meters(value.x), km_to_meters(value.y)),
            velocity: DVec2::new(
                km_per_s_to_meters_per_second(value.vx),
                km_per_s_to_meters_per_second(value.vy),
            ),
            color: HORIZONS_COLORS.get(&value.name).copied().unwrap_or(GRAY),
            name: value.name.clone(),
        };
        system.push(new_body);
    }

    let mut asteroids_added = 0;

    if take_user_choice("Add fake asteroids? ") {
        asteroids_added += initialize_bodies_spiro(
            &1200,
            &system.len(),
            &(2.5 * AU),
            &10e12,
            &LIGHTGRAY,
            &(25e5),
            &1.,
            system,
            &0.,
            Variance::WithVariance(0.8, 1.8),
            Variance::WithVariance(0.8, 1.2),
            &CenterObjectExists(system[0].mass, system[0].position),
            "Asteroids",
        )
        .0
    }

    (
        horizons_values.len() + asteroids_added,
        horizons_values.len(),
    )
}

fn km_to_meters(distance_km: f64) -> f64 {
    distance_km * 1000.
}

fn km_per_s_to_meters_per_second(velocity_km_p_s: f64) -> f64 {
    velocity_km_p_s * 1000.
}
