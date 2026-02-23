
use crate::constants::*;
use macroquad::{color, color::Color, math::DVec2};
use std::fs::File;
use csv::Writer;
use macroquad::color::RED;
use macroquad::prelude::draw_circle;
use rayon::iter::{IntoParallelIterator, ParallelIterator};


static WINDOW_FACTOR: f64 = (SCREEN_SIZE as f64) / (SCALING_FACTOR * EARTH_ORBITAL_RADIUS);





pub struct Particle {
    //Particle struct representing different values of bodies being simulated
    pub mass: f64, //kg
    pub position: DVec2, // In meters
    pub velocity: DVec2, // In meters/second
    pub radius: f64, // In meters
    pub visible_radius: f32, // In pixels
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
            if body_number != self_index && system[body_number].mass > 0.0 {
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
            if system[body_number].mass > 0.0 {
                let distance: f64 = (system[body_number].position - self.position).length();
                energy += -G * self.mass * system[body_number].mass / (distance);
            }
        }

        energy
    }

    pub fn update_kinetic_energy(&mut self) {
        self.kinetic_energy = 0.5 * self.velocity.length().powf(2.) * self.mass;
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
            if (system[i].position - system[j].position).length() < system[i].radius + system[j].radius && system[j].mass > 0.0 && system[j].mass > 0.0{
                let total_mass = system[i].mass + system[j].mass;
                system[j].position = (system[i].position * system[i].mass + system[j].position * system[j].mass) / total_mass;
                system[j].velocity = (system[i].velocity * system[i].mass + system[j].velocity * system[j].mass) / total_mass;
                system[i].radius = (system[i].radius.powi(3) + system[j].radius.powi(3)).cbrt();

                number_of_collisions += 1;
                println!("Collision!!!!");

                system[i].mass += system[j].mass;
                system[i].color = RED;
                system[i].visible_radius = system[i].visible_radius * 2.;
                
                system[j].mass = 0.0;
                system[j].visible_radius = 0.0;
                system[j].position = COLLIDED_POSITION;
            }
        }
    }
    number_of_collisions
}



pub fn add_physical_data(system: &[Particle; NUMBER_OF_BODIES], time: f64, wtr: &mut Writer<File>, rows: usize) {

    let mut newline: [String; NUMBER_OF_BODIES * 2 + 4] = std::array::from_fn(|_| String::from(""));

    newline[0] = time.to_string();

    for i in 0..NUMBER_OF_BODIES {
        newline[2 * i + 4] = system[i].position[0].to_string();
        newline[2 * i + 5] = system[i].position[1].to_string();
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

    let mut newline: [String; NUMBER_OF_BODIES * 2 + 4] = std::array::from_fn(|_| String::from(""));
    newline[0] = String::from("Time");

    newline[1] = String::from("Kinetic Energy");
    newline[2] = String::from("Gravitational Potential Energy");
    newline[3] = String::from("Total Energy");


    for i in 0..NUMBER_OF_BODIES {
        newline[2 * i + 4] = system[i].name.clone();
    }
    wtr.write_record(newline).unwrap();



    let mut newline: [String; NUMBER_OF_BODIES * 2 + 4] = std::array::from_fn(|_| String::from(""));
    for i in 0..NUMBER_OF_BODIES {
        newline[2 * i + 4] = String::from("Mass");
        newline[2 * i + 5] = system[i].mass.to_string();
    }
    wtr.write_record(newline).unwrap();


    let mut newline: [String; NUMBER_OF_BODIES * 2 + 4] = std::array::from_fn(|_| String::from(""));
    for i in 0..NUMBER_OF_BODIES {
        newline[2 * i + 4] = String::from("Position in X");
        newline[2 * i + 5] = String::from("Position in Y");
    }
    wtr.write_record(newline).unwrap();

    wtr.flush().unwrap();

}

pub fn draw_bodies(system: &[Particle; NUMBER_OF_BODIES]) {
    for i in 0..NUMBER_OF_BODIES {
        draw_circle(scale_window(system[i].position[0]),
                    scale_window(system[i].position[1]),
                    system[i].visible_radius,
                    system[i].color);
    }
}



pub fn velocity_to_color(velocity: DVec2) -> Color {
    let hue: f32 = 0.0f32.max(0.72f32.min(0.6*(velocity.length() / EARTH_ORBITAL_VELOCITY).powf(2.) as f32));
    color::hsl_to_rgb(hue, 1., 0.5)
}
