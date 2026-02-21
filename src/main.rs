use macroquad::{prelude::*};
use rayon::prelude::*;
use macroquad::rand::gen_range;
use std::fs::File;
mod constants;
use constants::*;


mod helpers;
use helpers::*;



static CENTER_COORDS: DVec2 = DVec2::new(SCALING_FACTOR * 0.5 * EARTH_ORBITAL_RADIUS, SCALING_FACTOR *0.5* EARTH_ORBITAL_RADIUS);


fn gravity_conf() -> Conf {
      Conf {
      window_title:"Gravity_Sim".to_owned(),
      window_height: SCREEN_SIZE,
      window_width: SCREEN_SIZE,
      ..Default::default()
    }
}
#[macroquad::main(gravity_conf)]
async fn main() {
    // Creating the array of particles representing the system with blank values at first
    let mut system: [Particle; NUMBER_OF_BODIES] = std::array::from_fn(|_| Particle {
        mass: 0.0,
        position: DVec2::new(0., 0.),
        velocity: DVec2::new(0., 0.),
        radius: 0.,
        visible_radius: 0.,
        color: WHITE,
        name: String::from("Default")
    });
    let star: Particle = Particle {
        mass: STAR_MASS,
        position: DVec2::new(CENTER_COORDS[0], CENTER_COORDS[1]-0.5*EARTH_ORBITAL_RADIUS),
        velocity: DVec2::new(0.5*EARTH_ORBITAL_VELOCITY, 0.),
        radius: scale(STAR_RADIUS),
        visible_radius: STAR_VISIBLE_RADIUS,
        color: DARKPURPLE,
        name: String::from("Sun1")
    };
    system[0] = star;

    let planet: Particle = Particle {
        mass: STAR_MASS,
        position: DVec2::new(CENTER_COORDS[0], CENTER_COORDS[1]+0.5*EARTH_ORBITAL_RADIUS),
        velocity: DVec2::new(-0.5*EARTH_ORBITAL_VELOCITY, 0.),
        radius: scale(STAR_RADIUS),
        visible_radius: STAR_VISIBLE_RADIUS,
        color: DARKPURPLE,
        name: String::from("Sun2")
    };    
    system[1] = planet;

    let gamma: f64 = 2.0 * std::f64::consts::PI;

    // Generates a number of comets with varying masses, positions, and velocities
    for i in 2..NUMBER_OF_BODIES {
        let comet_orbital_radius: f64 = gen_range(COMET_VARIANCE_MIN, COMET_VARIANCE_MAX) * COMET_ORBITAL_RADIUS;
        let angular_position: f64 = gamma * gen_range(0.0, 1.0);
        let comet_x_position: f64 = CENTER_COORDS[0] + angular_position.cos() * comet_orbital_radius;
        let comet_y_position: f64 = CENTER_COORDS[1] + angular_position.sin() * comet_orbital_radius;
        let comet_position: DVec2 = DVec2::new(comet_x_position, comet_y_position);

        let velocity_direction: f64 = angular_position + 0.5 * std::f64::consts::PI;
        let comet_x_velocity: f64 = velocity_direction.cos() * COMET_ORBITAL_VELOCITY;
        let comet_y_velocity: f64 = velocity_direction.sin() * COMET_ORBITAL_VELOCITY;

        let comet_angular_velocity: DVec2 = DVec2::new(comet_x_velocity, comet_y_velocity);

        let comet_mass: f64 = gen_range(COMET_VARIANCE_MIN, COMET_VARIANCE_MIN) * COMET_MASS;
        let new_comet: Particle = Particle {
            mass: comet_mass,
            position: comet_position,
            velocity: comet_angular_velocity,
            radius: COMET_RADIUS,
            visible_radius: COMET_VISIBLE_RADIUS,
            color: BLUE,
            name: String::from("Comet")
        };

        system[i] = new_comet;
    }

    // This ticker will count the amount of frames multiplied by the number of bodies
    let mut ticker: usize = 0;

    let mut seconds_passed_in_sim: f64 = 0.;


    // old_positions stores for a decided amount of frames the past the positions of all bodies to draw later
    let mut old_positions: [Vec2; OLD_FRAME_LIMIT*NUMBER_OF_BODIES] = [Vec2::new(0.,0.); OLD_FRAME_LIMIT*NUMBER_OF_BODIES];


    let my_file = File::create("my_file.csv").unwrap();


    add_topline_data(&system, &my_file);

    add_physical_data(&system, &my_file, seconds_passed_in_sim);

    draw_bodies(&system);
    let mut elapsed = 0.0;
    while elapsed < 10.0 {
        elapsed += get_frame_time();
        draw_bodies(&system);
        next_frame().await;
    }
    loop {
        clear_background(BLACK);




        // Parallel calculation of all the forces acting on the bodies using the calculate_g_force method
        let forces: Vec<DVec2> = (0..NUMBER_OF_BODIES)
            .into_par_iter()
            .map(|i| system[i].calculate_g_force(&system, i))
            .collect();

        // Applies forces to the system
        for i in 0..NUMBER_OF_BODIES {
            system[i].g_accelerate(forces[i]);
        }


        // Adds new positions to old_position and iterates ticker
        for i in 0..NUMBER_OF_BODIES {
            let old_pos_x = scale(system[i].position[0]) as f32;
            let old_pos_y = scale(system[i].position[1]) as f32;
            old_positions[ticker % (NUM_OLD_POSITION_LIMIT)] = Vec2::new(old_pos_x, old_pos_y);
            ticker += 1;
        }

        // Draws the trail using old_positions
        let draw_count = ticker.min(NUM_OLD_POSITION_LIMIT);
        for i in 0..draw_count {
            draw_circle(old_positions[i][0], old_positions[i][1], TRAIL_RADIUS, WHITE);
        }
        



        // Draws main bodies
        draw_bodies(&system);

        seconds_passed_in_sim += DT;
        let years_passed_in_sim: String = (seconds_passed_in_sim / SECONDS_IN_YEAR).to_string();
        let s = format!("Years Passed: {}", &years_passed_in_sim[0..5]);
        draw_text(&s, 10.0, 785.0, 30.0, WHITE);

        if (ticker / NUMBER_OF_BODIES) < ROW_LIMIT {
            add_physical_data(&system, &my_file, seconds_passed_in_sim);
        }

        next_frame().await
    }





}
