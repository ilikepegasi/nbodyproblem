use macroquad::prelude::*;
use macroquad::rand::gen_range;
use rayon::prelude::*;
use std::fs::File;
use std::f64::consts::TAU;

mod constants;
use constants::*;

mod helpers;
mod init_helpers;
use init_helpers::*;
use helpers::*;

// TODO: Refactor the generation of the bodies into their own function in helpers.rs

// TODO: Implement volume as a trait of Particle
// TODO: Implement scenarios: this would be spirograph/s, and I want to implement figure 8/f


fn gravity_conf() -> Conf {
    Conf {
        window_title: "Gravity_Sim".to_owned(),
        window_height: SCREEN_SIZE,
        window_width: SCREEN_SIZE,
        ..Default::default()
    }
}
#[macroquad::main(gravity_conf)]
async fn main() {
    let file_write = take_user_choice("Do you want to write to a file? ");
    let trails = take_user_choice("Do you want to have trails? ");
    let collisions = take_user_choice("Do you want to have collisions? ");
    let mut total_bodies_added = 0;

    let mut num_important_bodies = 0;

    // Creating the array of particles representing the system with blank values at first
    let mut system: [Particle; NUMBER_OF_BODIES] = std::array::from_fn(|_| Particle {
        mass: 0.0,
        position: DVec2::new(0., 0.),
        velocity: DVec2::new(0., 0.),
        radius: 0.,
        color: WHITE,
        name: String::from("Default"),
        kinetic_energy: 0.,
    });
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
    system[0] = star;
    num_important_bodies += 1;

    initialize_bodies_spiro();


    // Generates a number of comets with varying masses, positions, and velocities
    for i in EARTH_NUMBER + 1..NUMBER_OF_BODIES {
        let comet_orbital_radius: f64 = gen_range(
            COMET_ORBITAL_RADIUS_VARIANCE_MIN,
            COMET_ORBITAL_RADIUS_VARIANCE_MAX,
        ) * COMET_ORBITAL_RADIUS;
        let angular_position: f64 = gamma * gen_range(0.0, 1.0);
        let comet_x_position: f64 =
            CENTER_COORDS[0] + angular_position.cos() * comet_orbital_radius;
        let comet_y_position: f64 =
            CENTER_COORDS[1] + angular_position.sin() * comet_orbital_radius;
        let comet_position: DVec2 = DVec2::new(comet_x_position, comet_y_position);

        let velocity_direction: f64 = angular_position + 0.5 * std::f64::consts::PI;
        let orbital_speed = calculate_orbital_speed(&system[0], comet_position);
        let comet_x_velocity: f64 = velocity_direction.cos() * orbital_speed;
        let comet_y_velocity: f64 = velocity_direction.sin() * orbital_speed;

        let comet_velocity: DVec2 = DVec2::new(comet_x_velocity, comet_y_velocity);

        let comet_mass: f64 =
            gen_range(COMET_MASS_VARIANCE_MIN, COMET_MASS_VARIANCE_MAX) * COMET_MASS;
        let mut new_comet: Particle = Particle {
            mass: comet_mass,
            position: comet_position,
            velocity: comet_velocity,
            radius: COMET_RADIUS,
            color: LIGHTGRAY,
            name: String::from(format!("Comet {}", i)),
            kinetic_energy: 0.,
        };
        new_comet.update_kinetic_energy();
        system[i] = new_comet;
    }

    let mut collision_counter: u32 = 0;

    // This ticker will count the amount of frames multiplied by the number of bodies
    let mut trail_point_counter: usize = 0;
    let mut total_physics_ticks: usize = 0;
    let mut seconds_passed_in_sim: f64 = 0.0;

    let minimum_speed_color = calculate_orbital_speed(
        &system[0].mass,
        &system[0].position,
        DVec2::new(0., MIN_RADIUS_COLOR),
    ).log10() as f32 + 0.5;
    let maximum_speed_color = calculate_orbital_speed(
        &system[0].mass,
        &system[0].position,
        DVec2::new(0., MAX_RADIUS_COLOR),
    ).log10() as f32 - 0.5;

    // old_positions stores for a decided amount of frames the past the positions of all bodies to draw later
    let mut trail_values = vec![
        vec![vec![DVec2::new(0., 0.), DVec2::new(0., 0.)]; OLD_FRAME_LIMIT];
        num_important_bodies
    ];

    let my_file = if file_write {
        Some(File::create("orbital_simulation.csv").unwrap())
    } else {
        None
    };
    let mut rows_added = 0;

    let mut wtr = my_file.map(|f| csv::Writer::from_writer(f));
    if file_write {
        if let Some(ref mut w) = wtr {
            add_topline_data(&system, w);
            add_physical_data(&system, seconds_passed_in_sim, w, rows_added);
        }
        rows_added += 1;
    }
    draw_bodies(&system);
    let mut frames_waited = 0;
    while frames_waited < 0 * (1. / VIEWER_SECONDS_PER_FRAME) as i32 {
        draw_bodies(&system);
        next_frame().await;
        frames_waited += 1;
    }

    loop {
        clear_background(BLACK);

        for _i in 0..TICKS_PER_FRAME {
            total_physics_ticks += 1;
            /* Parallel calculation of all the forces acting on the bodies using the
            calculate_g_force method */
            let forces: Vec<DVec2> = (0..NUMBER_OF_BODIES)
                .into_par_iter()
                .map(|i| system[i].calculate_g_force(&system, i))
                .collect();

            // Applies forces to the system
            for i in 0..NUMBER_OF_BODIES {
                system[i].kick(forces[i]);
            }
            for i in 0..NUMBER_OF_BODIES {
                system[i].drift();
            }
            let forces: Vec<DVec2> = (0..NUMBER_OF_BODIES)
                .into_par_iter()
                .map(|i| system[i].calculate_g_force(&system, i))
                .collect();
            for i in 0..NUMBER_OF_BODIES {
                system[i].kick(forces[i]);
                system[i].update_kinetic_energy();
            }
            if collisions {
                collision_counter += collision_engine(&mut system)
            };
            if file_write {
                if let Some(ref mut w) = wtr {
                    if rows_added < ROW_LIMIT && total_physics_ticks % DATA_INTERVAL == 0 {
                        add_physical_data(&system, seconds_passed_in_sim, w, rows_added);
                        rows_added += 1;
                    }
                }
            }
            seconds_passed_in_sim += DT;
        }


        if trails {
            draw_trails(
                num_important_bodies,
                &system,
                &mut trail_point_counter,
                &mut trail_values,
                minimum_speed_color,
                maximum_speed_color,
            );
        }

        // Draws main bodies
        draw_bodies(&system);

        // Helper circle around Earth's orbit
        draw_poly_lines(
            scale_window(CENTER_COORDS[0]),
            scale_window(CENTER_COORDS[1]),
            64,
            scale_window(EARTH_ORBITAL_RADIUS),
            0.,
            1.,
            RED,
        );

        let years_passed_in_sim: String = (seconds_passed_in_sim / SECONDS_IN_YEAR).to_string();
        let mut info_on_screen = format!(
            "Years Passed: {:.5}/{:.2} | Total Physics Ticks: {}",
            &years_passed_in_sim,
            &YEARS_OF_WRITING,
            total_physics_ticks
        );
        draw_text(
            &info_on_screen,
            10.0,
            (SCREEN_SIZE - 80) as f32,
            20.0,
            WHITE,
        );
        if file_write {
            info_on_screen.push_str(&format!(
                " | Still Writing: {} (with {} rows)",
                rows_added < ROW_LIMIT,
                rows_added
            ));
        }
        if collision_counter > 0 {
            info_on_screen.push_str(&format!(" | Collision Count: {}", collision_counter));
        }

        draw_text(
            &info_on_screen,
            10.0,
            (SCREEN_SIZE - 80) as f32,
            20.0,
            WHITE,
        );
        draw_fps();
        next_frame().await
    }
}