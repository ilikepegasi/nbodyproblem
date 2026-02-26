use macroquad::prelude::*;
use rayon::prelude::*;
use std::f64::consts::TAU;
use std::fs::File;

mod constants;
use constants::*;

mod helpers;
mod init_helpers;
use helpers::*;
use init_helpers::*;

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
    let scenario_key_list: Vec<ScenarioKey> = vec!(ScenarioKey::Scenario("Spirograph".to_string(), 0), ScenarioKey::Scenario("Figure 8".to_string(), 1));



    let file_write = take_user_choice("Do you want to write to a file? ");
    let trails = take_user_choice("Do you want to have trails? ");
    let collisions = take_user_choice("Do you want to have collisions? ");
    let mut names_of_scenarios: String = "".to_string();
    for ScenarioKey::Scenario(a, b) in &scenario_key_list {
    names_of_scenarios.push_str(&format!("\n[{}] {} Scenario", b, a));
    }
    let scenario: usize = get_number_from_user(format!("What scenario to use? {}", names_of_scenarios).as_str()) as usize;
    let mut total_bodies_added = 0;
    let mut num_important_bodies = 0;
    let mut system: Vec<Particle> = Vec::new();
    let init_output = initialize_from_scenario(scenario, &mut system, &scenario_key_list);
    total_bodies_added += init_output.0;
    num_important_bodies += init_output.1;
    
    let dt = init_output.2;
    let data_interval: usize = (SIM_SECONDS_PER_DATA_ROW / dt) as usize;


    assert_eq!(total_bodies_added, system.len());
    // Generates a number of comets with varying masses, positions, and velocities

    let mut collision_counter: u32 = 0;

    // This ticker will count the amount of frames multiplied by the number of bodies
    let mut trail_point_counter: usize = 0;
    let mut total_physics_ticks: usize = 0;
    let mut seconds_passed_in_sim: f64 = 0.0;

    let minimum_speed_color = calculate_orbital_speed(
        &system[0].mass,
        &system[0].position,
        DVec2::new(
            system[0].position.x,
            system[0].position.y + MAX_RADIUS_MIN_COLOR,
        ),
    )
    .log10() as f32;
    let maximum_speed_color = calculate_orbital_speed(
        &system[0].mass,
        &system[0].position,
        DVec2::new(
            system[0].position.x,
            system[0].position.y + MIN_RADIUS_MAX_COLOR,
        ),
    )
    .log10() as f32;

    // old_positions stores for a decided amount of frames the past the positions of all bodies to draw later
    let mut trail_values: Vec<Vec<(DVec2, Color)>> = vec![
        vec![(DVec2::new(0., 0.), WHITE); OLD_FRAME_LIMIT];
        num_important_bodies];

    let my_file = if file_write {
        Some(File::create(format!("orbital_simulation_{}.csv", TICKS_PER_FRAME)).unwrap())
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
    let mut time_to_wait = get_number_from_user("How long to wait?");
    while time_to_wait > 0.0 {
        draw_bodies(&system);
        next_frame().await;
        time_to_wait -= get_frame_time().min(0.1);
    }

    loop {
        clear_background(BLACK);

        for _i in 0..TICKS_PER_FRAME {
            total_physics_ticks += 1;
            /* Parallel calculation of all the forces acting on the bodies using the
            calculate_g_force method */
            let forces: Vec<DVec2> = (0..system.len())
                .into_par_iter()
                .map(|i| system[i].calculate_g_force(&system, i))
                .collect();

            // Applies forces to the system
            for i in 0..system.len() {
                system[i].kick(forces[i], dt);
            }
            for i in 0..system.len() {
                system[i].drift(dt);
            }
            let forces: Vec<DVec2> = (0..system.len())
                .into_par_iter()
                .map(|i| system[i].calculate_g_force(&system, i))
                .collect();
            for i in 0..system.len() {
                system[i].kick(forces[i], dt);
                system[i].update_kinetic_energy();
            }
            if collisions {
                collision_counter += collision_engine(&mut system)
            };
            if file_write {
                if let Some(ref mut w) = wtr {
                    if rows_added < ROW_LIMIT && total_physics_ticks % data_interval == 0 {
                        add_physical_data(&system, seconds_passed_in_sim, w, rows_added);
                        rows_added += 1;
                    }
                }
            }
            seconds_passed_in_sim += dt;
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
            &years_passed_in_sim, &YEARS_OF_WRITING, total_physics_ticks
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
