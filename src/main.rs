use macroquad::{prelude::*};
use rayon::prelude::*;
use macroquad::rand::gen_range;
use std::fs::File;
mod constants;
use constants::*;


mod helpers;
use helpers::*;





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
        name: String::from("Default"),
        kinetic_energy: 0.
    });
    let mut star: Particle = Particle {
        mass: STAR_MASS,
        position: DVec2::new(CENTER_COORDS[0], CENTER_COORDS[1]),
        velocity: DVec2::new(-0.1*EARTH_ORBITAL_VELOCITY, 0.),
        radius: STAR_RADIUS,
        visible_radius: STAR_VISIBLE_RADIUS,
        color: YELLOW,
        name: String::from("Sun"),
        kinetic_energy: 0.
    };
    star.update_kinetic_energy();
    system[0] = star;

    let mut planet: Particle = Particle {
        mass: 0.25*STAR_MASS,
        position: DVec2::new(CENTER_COORDS[0], CENTER_COORDS[1]-EARTH_ORBITAL_RADIUS),
        velocity: DVec2::new(0.4*EARTH_ORBITAL_VELOCITY, 0.),
        radius: EARTH_RADIUS,
        visible_radius: PLANET_VISIBLE_RADIUS,
        color: BLUE,
        name: String::from("Earth"),
        kinetic_energy: 0.
    };
    planet.update_kinetic_energy();
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
        let orbital_speed = calculate_orbital_speed(&system[0], comet_position);
        let comet_x_velocity: f64 = velocity_direction.cos() * orbital_speed;
        let comet_y_velocity: f64 = velocity_direction.sin() * orbital_speed;

        let comet_velocity: DVec2 = DVec2::new(comet_x_velocity, comet_y_velocity);

        let comet_mass: f64 = gen_range(COMET_VARIANCE_MIN, COMET_VARIANCE_MAX) * COMET_MASS;
        let mut new_comet: Particle = Particle {
            mass: comet_mass,
            position: comet_position,
            velocity: comet_velocity,
            radius: COMET_RADIUS,
            visible_radius: SMALL_OBJECT_VISIBLE_RADIUS,
            color: LIGHTGRAY,
            name: String::from(format!("Comet {}", i)),
            kinetic_energy: 0.
        };
        new_comet.update_kinetic_energy();
        system[i] = new_comet;
    }

    // This ticker will count the amount of frames multiplied by the number of bodies
    let mut ticker: usize = 0;

    let mut seconds_passed_in_sim: f64 = 0.;


    // old_positions stores for a decided amount of frames the past the positions of all bodies to draw later
    let mut old_positions: [Vec2; NUM_OLD_POSITION_LIMIT] =
        [Vec2::new(0.,0.); NUM_OLD_POSITION_LIMIT];


    let my_file = File::create("my_file.csv").unwrap();

    let mut wtr = csv::Writer::from_writer(my_file);

    add_topline_data(&system, &mut wtr);
    let mut rows_added = 0;

    add_physical_data(&system, seconds_passed_in_sim, &mut wtr, rows_added);
    rows_added += 1;
    draw_bodies(&system);
    let mut elapsed = 0.0;
    while elapsed < 1.0 {
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
            system[i].accelerate(forces[i]);
            system[i].update_kinetic_energy();
        }


        // Adds new positions to old_position and iterates ticker
        if TRAILS {
            for i in 0..NUMBER_OF_BODIES {
                let old_pos_x = scale_window(system[i].position[0]) as f32;
                let old_pos_y = scale_window(system[i].position[1]) as f32;
                old_positions[ticker % (NUM_OLD_POSITION_LIMIT)] = Vec2::new(old_pos_x, old_pos_y);
                ticker += 1;
            }
            // Draws the trail using old_positions
            let draw_count = ticker.min(NUM_OLD_POSITION_LIMIT);
            for i in 0..draw_count {
                draw_circle(old_positions[i][0], old_positions[i][1], TRAIL_RADIUS, WHITE);
            }
        }


        



        // Draws main bodies
        draw_bodies(&system);


        draw_poly_lines(scale_window(CENTER_COORDS[0]) as f32,
                        scale_window(CENTER_COORDS[1]) as f32,
                        64,
                        scale_window(EARTH_ORBITAL_RADIUS) as f32,
                        0.,
                        1.,
                        RED);

        seconds_passed_in_sim += DT;
        let years_passed_in_sim: String = (seconds_passed_in_sim / SECONDS_IN_YEAR).to_string();
        let s = format!("Years Passed: {} | Still Writing: {} (with {} rows)",
                        &years_passed_in_sim[0..5],
                        rows_added < ROW_LIMIT,
                        rows_added);
        draw_text(&s, 10.0, 790.0, 20.0, WHITE);

        if rows_added < ROW_LIMIT && FILE_WRITE{
            add_physical_data(&system, seconds_passed_in_sim, &mut wtr, rows_added);
            rows_added += 1;
        }

        next_frame().await
    }





}
