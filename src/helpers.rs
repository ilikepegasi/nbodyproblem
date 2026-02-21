
use crate::constants::*;
use macroquad::{color::Color, math::DVec2};
use std::fs::File;
use macroquad::prelude::draw_circle;

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
}

impl Particle {
    // Method changing the values of a particle based on a given force vector

    pub fn accelerate(&mut self, force_vector: DVec2) {
        let acceleration = force_vector / self.mass;
        self.velocity += acceleration * DT;
        self.position += self.velocity * DT;
    }

    // Method calculating total force acting upon a body from the input of the array of all of the system's bodies
    pub fn calculate_g_force(&self, system: &[Particle; NUMBER_OF_BODIES], identity: usize) -> DVec2 {
        let mut force_vector = DVec2::new(0.,0.);


        for body_number in 0..NUMBER_OF_BODIES {
            // I'm using "identity" to make sure that the force from itself on itself isn't being calculated
            if body_number != identity {
                let distance: f64 = (system[body_number].position - self.position).length();
                let direction: DVec2 = (system[body_number].position - self.position) / (distance);

                let force_magnitude: f64 = G * ((system[body_number].mass * self.mass) / (distance*distance + 1e-10));
                force_vector += direction * force_magnitude;

            }
        }
        force_vector
    }

}

pub fn scale(distance: f64) -> f64 {
    distance * WINDOW_FACTOR
}


pub fn add_physical_data(system: &[Particle; NUMBER_OF_BODIES], my_file: &File, time: f64) {
    let mut wtr = csv::Writer::from_writer(my_file);

    let mut newline: [String; NUMBER_OF_BODIES * 2 + 1] = std::array::from_fn(|_| String::from(""));

    newline[0] = time.to_string();

    for i in 0..NUMBER_OF_BODIES {
        newline[2 * i + 1] = system[i].position[0].to_string();
        newline[2 * i + 2] = system[i].position[1].to_string();
    }
    wtr.write_record(newline).unwrap();
    wtr.flush().unwrap();
}

pub fn add_topline_data(system: &[Particle; NUMBER_OF_BODIES], my_file: &File) {
    let mut wtr = csv::Writer::from_writer(my_file);

    let mut newline: [String; NUMBER_OF_BODIES * 2 + 1] = std::array::from_fn(|_| String::from(""));

    for i in 0..NUMBER_OF_BODIES {
        newline[2 * i + 1] = system[i].name.clone();
    }
    wtr.write_record(newline).unwrap();



    let mut newline: [String; NUMBER_OF_BODIES * 2 + 1] = std::array::from_fn(|_| String::from(""));
    for i in 0..NUMBER_OF_BODIES {
        newline[2 * i + 1] = String::from("Mass");
        newline[2 * i + 2] = system[i].mass.to_string();
    }
    wtr.write_record(newline).unwrap();


    let mut newline: [String; NUMBER_OF_BODIES * 2 + 1] = std::array::from_fn(|_| String::from(""));
    for i in 0..NUMBER_OF_BODIES {
        newline[2 * i + 1] = String::from("Position in X");
        newline[2 * i + 2] = String::from("Position in Y");
    }
    wtr.write_record(newline).unwrap();

    wtr.flush().unwrap();

}

pub fn draw_bodies(system: &[Particle; NUMBER_OF_BODIES]) {
    for i in 0..NUMBER_OF_BODIES {
        draw_circle(scale(system[i].position[0]) as f32,
                    scale(system[i].position[1]) as f32,
                    system[i].visible_radius,
                    system[i].color);
    }
}

