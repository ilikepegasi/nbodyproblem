
use crate::constants::*;
use macroquad::{color, color::Color, math::DVec2};
use std::fs::File;
use std::io;
use csv::Writer;
use macroquad::prelude::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};







pub struct Particle {
    //Particle struct representing different values of bodies being simulated
    pub mass: f64, //kg
    pub position: DVec2, // In meters
    pub velocity: DVec2, // In meters/second
    pub radius: f64, // In meters
    pub color: Color,
    pub name: String,
    pub kinetic_energy: f64,
}

impl Particle {
    // Method changing the values of a particle based on a given force vector

    pub fn drift(&mut self) {
        self.position += self.velocity * DT;
    }
    pub fn kick(&mut self, force: DVec2) {
        let acceleration = force / self.mass;
        self.velocity += 0.5 * acceleration * DT;
    }


    // Method calculating total force acting upon a body from the input of the array of
    // all the system's bodies
    pub fn calculate_g_force(&self, system: &[Particle; NUMBER_OF_BODIES], self_index: usize) -> DVec2 {
        let mut force_vector = DVec2::new(0.,0.);


        for body_number in 0..NUMBER_OF_BODIES {
            // I'm using "self_index" to make sure that the force from itself on itself isn't being calculated
            if body_number != self_index && system[body_number].mass > MIN_MASS && self.mass > MIN_MASS {
                let distance: f64 = (system[body_number].position - self.position).length();
                let direction: DVec2 = (system[body_number].position - self.position) / (distance);

                let force_magnitude: f64 = G * ((system[body_number].mass * self.mass) / (distance*distance + EPSILON * EPSILON));
                force_vector += direction * force_magnitude;

            }
        }
        force_vector
    }
    pub fn find_potential_gravitational_energy(&self, system: &[Particle; NUMBER_OF_BODIES], self_index: usize) -> f64 {
        let mut energy: f64 = 0.;

        for body_number in (self_index+1)..NUMBER_OF_BODIES {
            if system[body_number].mass > MIN_MASS {
                let distance: f64 = (system[body_number].position - self.position).length();
                energy += -G * self.mass * system[body_number].mass / (distance);
            }
        }

        energy
    }

    pub fn update_kinetic_energy(&mut self) {
        self.kinetic_energy = 0.5 * self.velocity.length().powf(2.) * self.mass;
    }

    pub fn generate_visible_radius(&self) -> f32 {
        let log_min = SMALL_RADIUS.log10() as f32;
        let log_max = STAR_RADIUS.log10() as f32;
        let log_radius = self.radius.log10() as f32;

        let radius_scale = ((log_radius - log_min) / (log_max - log_min)).clamp(0.0, 1.0);
        MIN_RADIUS_PIXELS + radius_scale * (MAX_RADIUS_PIXELS - MIN_RADIUS_PIXELS)
    }

}

pub fn calculate_orbital_speed(center_object: &Particle, position: DVec2) -> f64 {
    let distance = (center_object.position - position).length();
    let speed = ((G * center_object.mass) / distance).sqrt();
    speed
}


// This is used to correctly translate from coords in the simulation system into coords for graphics
pub fn scale_window(distance: f64) -> f32 {
    (distance * WINDOW_FACTOR) as f32
}




pub fn find_system_kinetic_energy(system: &[Particle; NUMBER_OF_BODIES]) -> f64 {
    let mut total_energy: f64 = 0.;
    for i in 0..NUMBER_OF_BODIES {
        if system[i].mass != 0.0 {
            total_energy += system[i].kinetic_energy;
        }
    }
    total_energy
}

pub fn collision_engine(system: &mut [Particle; NUMBER_OF_BODIES]) -> u32 {
    let mut number_of_collisions: u32 = 0;
    for i in 0..NUMBER_OF_BODIES {
        for j in i+1..NUMBER_OF_BODIES {
            if (system[i].position - system[j].position).length() < system[i].radius + system[j].radius && system[j].mass > MIN_MASS && system[j].mass > MIN_MASS{
                let total_mass = system[i].mass + system[j].mass;
                system[j].position = (system[i].position * system[i].mass + system[j].position * system[j].mass) / total_mass;
                system[j].velocity = (system[i].velocity * system[i].mass + system[j].velocity * system[j].mass) / total_mass;
                let density_i = system[i].mass / system[i].radius.powi(3);
                let density_j = system[j].mass / system[j].radius.powi(3);
                let merged_density = (density_i * system[i].mass + density_j * system[j].mass) / total_mass;
                let total_mass = system[i].mass + system[j].mass;
                let new_position = (system[i].position * system[i].mass + system[j].position * system[j].mass) / total_mass;
                let new_velocity = (system[i].velocity * system[i].mass + system[j].velocity * system[j].mass) / total_mass;

                system[i].position = new_position;  // Apply to survivor (i), not j
                system[i].velocity = new_velocity;
                system[i].radius = (system[i].radius.powi(3) + system[j].radius.powi(3)).cbrt();
                system[i].mass = total_mass;
                system[i].color = RED;

                system[j].mass = 0.0;
                system[j].radius = 0.0;
                system[j].position = COLLIDED_POSITION;
            }
        }
    }
    number_of_collisions
}



pub fn add_physical_data(system: &[Particle; NUMBER_OF_BODIES], time: f64, wtr: &mut Writer<File>, rows: usize) {

    let mut newline: [String; NUMBER_OF_BODIES * COLUMNS_PER_OBJECT + LEFT_PAD] = std::array::from_fn(|_| String::from(""));

    newline[0] = time.to_string();

    for i in 0..NUMBER_OF_BODIES {
        newline[COLUMNS_PER_OBJECT * i + LEFT_PAD] = system[i].position[0].to_string();
        newline[COLUMNS_PER_OBJECT * i + LEFT_PAD+1] = system[i].position[1].to_string();
        newline[COLUMNS_PER_OBJECT * i + LEFT_PAD+2] = system[i].mass.to_string();
    }

    if rows % ENERGY_INTERVAL == 0 {
        let total_kinetic_energy = find_system_kinetic_energy(system);
        let gravitational_energies: Vec<f64> = (0..NUMBER_OF_BODIES)
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

pub fn add_topline_data(system: &[Particle; NUMBER_OF_BODIES], wtr: &mut Writer<File>) {

    let mut newline: [String; NUMBER_OF_BODIES * COLUMNS_PER_OBJECT + LEFT_PAD] = std::array::from_fn(|_| String::from(""));
    newline[0] = String::from("Time");

    newline[1] = String::from("Kinetic Energy");
    newline[2] = String::from("Gravitational Potential Energy");
    newline[3] = String::from("Total Energy");






    let mut newline: [String; NUMBER_OF_BODIES * 2 + 4] = std::array::from_fn(|_| String::from(""));
    for i in 0..NUMBER_OF_BODIES {
        newline[COLUMNS_PER_OBJECT * i + LEFT_PAD] = String::from(format!("Mass of {}", system[i].name));
        newline[COLUMNS_PER_OBJECT * i + LEFT_PAD+1] = String::from(format!("Position in X of {}", system[i].name));
        newline[COLUMNS_PER_OBJECT * i + LEFT_PAD+2] = String::from(format!("Position in Y of {}", system[i].name));

    }
    wtr.write_record(newline).unwrap();
    wtr.flush().unwrap();
}

pub fn draw_bodies(system: &[Particle; NUMBER_OF_BODIES]) {
    for i in 0..NUMBER_OF_BODIES {
        draw_circle(scale_window(system[i].position[0]),
                    scale_window(system[i].position[1]),
                    system[i].generate_visible_radius(),
                    system[i].color);
    }
}



pub fn take_user_choice(question: &str) -> bool {
    let answer;
    let mut input = String::new();
    loop {
        println!("{}", question);
        input.clear();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        let trimmed = input.trim().to_lowercase();

        match trimmed.as_str() {
            "y" | "yes" => { answer = true; break; }
            "n" | "no"  => { answer = false; break; }
            _           => println!("Invalid input"),
        }
    }
    answer
}


pub fn velocity_to_color(velocity: DVec2) -> Color {
    let hue: f32 = 0.0f32.max(0.72f32.min(0.6*(velocity.length() / EARTH_ORBITAL_VELOCITY).powf(2.) as f32));
    color::hsl_to_rgb(hue, 1., 0.5)
}

pub fn draw_trails(num_important_bodies: usize, system: &[Particle; 1200], trail_point_counter: &mut usize, trail_values: &mut Vec<Vec<Vec<DVec2>>>) {
    for i in 0..num_important_bodies {
        trail_values[i][*trail_point_counter % OLD_FRAME_LIMIT][0] = system[i].position;
        trail_values[i][*trail_point_counter % OLD_FRAME_LIMIT][1] = system[i].velocity;
    }
    *trail_point_counter += 1;

    let recent_point = *trail_point_counter % OLD_FRAME_LIMIT;
    let gap_point = if recent_point == 0 { OLD_FRAME_LIMIT - 1 } else { recent_point - 1 };
    // Draws the trail using old_positions

    for i in 0..num_important_bodies {
        for j in 0..OLD_FRAME_LIMIT.min(*trail_point_counter) {
            if j != gap_point {
                let pos_1 = trail_values[i][j][0];
                let pos_2 = trail_values[i][(j + 1) % OLD_FRAME_LIMIT][0];
                if ((pos_2 - pos_1).length() as f32) < MAX_TRAIL_LINE_LEN {
                    draw_line(scale_window(pos_1[0]),
                              scale_window(pos_1[1]),
                              scale_window(pos_2[0]),
                              scale_window(pos_2[1]),
                              TRAIL_RADIUS,
                              velocity_to_color(trail_values[i][j][1]));
                }
            }
        }
    }
}