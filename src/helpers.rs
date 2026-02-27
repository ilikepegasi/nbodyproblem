use crate::constants::*;
use crate::init_helpers::*;

use csv::Writer;
use macroquad::prelude::*;
use macroquad::{color, color::Color, math::DVec2};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::fmt::Display;
use std::fs::File;
use std::io;


pub struct Particle {
    //Particle struct representing different values of bodies being simulated
    pub mass: f64,       //kg
    pub position: DVec2, // In meters
    pub velocity: DVec2, // In meters/second
    pub radius: f64,     // In meters
    pub color: Color,
    pub name: String,
}

impl Particle {
    // Method changing the values of a particle based on a given force vector

    pub fn drift(&mut self, dt: f64) {
        self.position += self.velocity * dt;
    }
    pub fn kick(&mut self, force: DVec2, dt: f64) {
        let acceleration = force / self.mass;
        self.velocity += 0.5 * acceleration * dt;
    }

    // Method calculating total force acting upon a body from the input of the array of
    // all the system's bodies
    pub fn calculate_g_force(&self, system: &Vec<Particle>, self_index: usize) -> DVec2 {
        let mut force_vector = DVec2::new(0., 0.);

        for body_number in 0..system.len() {
            // I'm using "self_index" to make sure that the force from itself on itself isn't being calculated
            if body_number != self_index
                && system[body_number].mass > COLLISION_MIN_MASS
                && self.mass > COLLISION_MIN_MASS
            {
                let distance: f64 = (system[body_number].position - self.position).length();
                let direction: DVec2 = (system[body_number].position - self.position) / (distance);

                let force_magnitude: f64 = G
                    * ((system[body_number].mass * self.mass)
                        / (distance * distance + EPSILON * EPSILON));
                force_vector += direction * force_magnitude;
            }
        }
        force_vector
    }
    pub fn find_potential_gravitational_energy(
        &self,
        system: &Vec<Particle>,
        self_index: usize,
    ) -> f64 {
        let mut energy: f64 = 0.;

        for body_number in (self_index + 1)..system.len() {
            if system[body_number].mass > COLLISION_MIN_MASS && self.mass > COLLISION_MIN_MASS {
                let distance: f64 = (system[body_number].position - self.position).length();
                energy += -G * self.mass * system[body_number].mass / (distance);
            }
        }

        energy
    }

    pub fn calculate_kinetic_energy(&self) -> f64 {
        0.5 * self.velocity.length().powf(2.) * self.mass
    }

    pub fn generate_visible_radius(&self) -> f32 {
        let log_min = SMALL_RADIUS.log10() as f32;
        let log_max = STAR_RADIUS.log10() as f32;
        let log_radius = self.radius.log10() as f32;

        let radius_scale = ((log_radius - log_min) / (log_max - log_min)).clamp(0.0, 1.0);
        MIN_RADIUS_PIXELS + radius_scale * (MAX_RADIUS_PIXELS - MIN_RADIUS_PIXELS)
    }
}

pub fn calculate_orbital_speed(
    center_object_mass: &f64,
    center_object_position: &DVec2,
    position: DVec2,
) -> f64 {
    let distance = (*center_object_position - position).length();
    let speed = ((*center_object_mass * G) / distance).sqrt();
    speed
}

// This is used to correctly translate from coords in the simulation system into coords for graphics
pub fn scale_window(distance: f64) -> f32 {
    (distance * WINDOW_FACTOR) as f32
}

pub fn find_system_kinetic_energy(system: &Vec<Particle>) -> f64 {
    let mut total_energy: f64 = 0.;
    for i in 0..system.len() {
        if system[i].mass != 0.0 {
            total_energy += system[i].calculate_kinetic_energy();
        }
    }
    total_energy
}

pub fn collision_engine(system: &mut Vec<Particle>) -> u32 {
    let mut number_of_collisions: u32 = 0;
    for i in 0..system.len() {
        for j in i + 1..system.len() {
            if (system[i].position - system[j].position).length()
                < system[i].radius + system[j].radius
                && system[i].mass > COLLISION_MIN_MASS
                && system[j].mass > COLLISION_MIN_MASS
            {
                let total_mass = system[i].mass + system[j].mass;
                let new_position = (system[i].position * system[i].mass
                    + system[j].position * system[j].mass)
                    / total_mass;
                let new_velocity = (system[i].velocity * system[i].mass
                    + system[j].velocity * system[j].mass)
                    / total_mass;
                let (collider_object, dead_object) = if system[i].mass >= system[j].mass {
                    (i, j)
                } else {
                    (j, i)
                };
                system[collider_object].position = new_position;
                system[collider_object].velocity = new_velocity;
                system[collider_object].radius = (system[collider_object].radius.powi(3)
                    + system[dead_object].radius.powi(3))
                .cbrt();
                if system[dead_object].mass > (0.1 * system[collider_object].mass) {
                    system[collider_object].color = PINK;
                };

                system[collider_object].mass = total_mass;

                system[dead_object].mass = 0.0;
                system[dead_object].radius = 0.0;
                system[dead_object].position = COLLIDED_POSITION;
                number_of_collisions += 1;
            }
        }
    }
    number_of_collisions
}

pub fn add_physical_data(system: &Vec<Particle>, time: f64, wtr: &mut Writer<File>, rows: usize) {
    let mut newline = vec!["".to_string(); system.len() * COLUMNS_PER_OBJECT + LEFT_PAD];

    newline[0] = time.to_string();

    for i in 0..system.len() {
        newline[COLUMNS_PER_OBJECT * i + LEFT_PAD] = system[i].position[0].to_string();
        newline[COLUMNS_PER_OBJECT * i + LEFT_PAD + 1] = system[i].position[1].to_string();
    }

    if rows % ENERGY_INTERVAL == 0 {
        let total_kinetic_energy = find_system_kinetic_energy(&system);
        let gravitational_energies: Vec<f64> = (0..system.len())
            .into_par_iter()
            .map(|i| system[i].find_potential_gravitational_energy(&system, i))
            .collect();
        let total_gravitational_energy: f64 = gravitational_energies.iter().sum();
        let total_energy: f64 = total_kinetic_energy + total_gravitational_energy;
        newline[1] = total_kinetic_energy.to_string();
        newline[2] = total_gravitational_energy.to_string();
        newline[3] = total_energy.to_string();
    } else {
        newline[1] = String::from("NaN");
        newline[2] = String::from("NaN");
        newline[3] = String::from("NaN");
    }

    wtr.write_record(newline).unwrap();
    wtr.flush().unwrap();
}

pub fn add_topline_data(system: &Vec<Particle>, wtr: &mut Writer<File>) {
    let mut newline = vec!["".to_string(); system.len() * COLUMNS_PER_OBJECT + LEFT_PAD];

    newline[0] = String::from("Time");
    newline[1] = String::from("Kinetic Energy");
    newline[2] = String::from("Gravitational Potential Energy");
    newline[3] = String::from("Total Energy");

    for i in 0..system.len() {
        newline[COLUMNS_PER_OBJECT * i + LEFT_PAD] = String::from(format!(
            "Position in X of {} with mass {:2e}",
            system[i].name, system[i].mass
        ));
        newline[COLUMNS_PER_OBJECT * i + LEFT_PAD + 1] = String::from(format!(
            "Position in Y of {} with mass {:2e}",
            system[i].name, system[i].mass
        ));
    }
    wtr.write_record(newline).unwrap();
    wtr.flush().unwrap();
}

pub fn draw_bodies(system: &Vec<Particle>) {
    for i in 0..system.len() {
        draw_circle(
            scale_window(system[i].position[0]),
            scale_window(system[i].position[1]),
            system[i].generate_visible_radius(),
            system[i].color,
        );
    }
}

pub fn take_user_choice(question: &str) -> bool {
    let answer;
    let mut input = String::new();
    loop {
        println!("{}", question);
        input.clear();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        let trimmed = input.trim().to_lowercase();

        match trimmed.as_str() {
            "y" | "yes" => {
                answer = true;
                break;
            }
            "n" | "no" => {
                answer = false;
                break;
            }
            _ => println!("Invalid input"),
        }
    }
    answer
}

pub fn velocity_to_color(velocity: DVec2, minimum_speed_log: f32, maximum_speed_log: f32) -> Color {
    let velocity_log: f32 = velocity.length().log10() as f32;

    let normalized = ((velocity_log - minimum_speed_log) / (maximum_speed_log - minimum_speed_log))
        .clamp(0.0, 1.0);

    let hue = (normalized) * MAX_VIOLET_HUE;

    color::hsl_to_rgb(hue, 0.4, 0.7)
}

pub fn draw_trails(
    num_important_bodies: usize,
    system: &Vec<Particle>,
    trail_point_counter: &mut usize,
    trail_values: &mut Vec<Vec<(DVec2, Color)>>,
    log_min_speed: f32,
    log_max_speed: f32,
    init_output: &ConfigValues,
) {
    for i in 0..num_important_bodies {
        trail_values[i][*trail_point_counter % init_output.trail_length].0 = system[i].position;
        trail_values[i][*trail_point_counter % init_output.trail_length].1 = velocity_to_color(system[i].velocity, log_min_speed, log_max_speed);
    }
    *trail_point_counter += 1;

    let recent_point = *trail_point_counter % init_output.trail_length;
    let gap_point = if recent_point == 0 {
        init_output.trail_length - 1
    } else {
        recent_point - 1
    };
    // Draws the trail using old_positions

    for i in 0..num_important_bodies {
        for j in 0..init_output.trail_length.min(*trail_point_counter) {
            if j != gap_point {
                let pos_1 = trail_values[i][j].0;
                let pos_2 = trail_values[i][(j + 1) % init_output.trail_length].0;
                if ((pos_2 - pos_1).length() as f32) < MAX_TRAIL_LINE_LEN {
                    draw_line(
                        scale_window(pos_1[0]),
                        scale_window(pos_1[1]),
                        scale_window(pos_2[0]),
                        scale_window(pos_2[1]),
                        TRAIL_RADIUS,
                        trail_values[i][j].1,
                    );
                }
            }
        }
    }
}

pub fn get_number_from_user(text: &str) -> f32 {
    loop {
        let mut user_input: String = String::new();
        println!("{}", text);
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");
        match user_input.trim().parse::<f32>() {
            Ok(number) => return number,
            Err(_) => println!("Invalid input. Please enter a valid number."),
        }
    }
}
pub fn get_pos_int_from_user(text: &str) -> u32 {
    loop {
        let mut user_input: String = String::new();
        println!("{}", text);
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");
        match user_input.trim().parse::<u32>() {
            Ok(number) => return number,
            Err(_) => println!("Invalid input. Please enter a valid number."),
        }
    }
}
