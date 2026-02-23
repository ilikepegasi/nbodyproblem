use macroquad::{prelude::*};
use rayon::prelude::*;
use macroquad::rand::gen_range;
use std::fs::File;
mod constants;
use constants::*;
use std::io;

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
    let file_write: bool;
    let mut input = String::new();
    loop {
        println!("Do you want to write results to a file? Yes/No:");
        input.clear();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        let trimmed = input.trim().to_lowercase();

        match trimmed.as_str() {
            "y" | "yes" => { file_write = true; break; }
            "n" | "no"  => { file_write = false; break; }
            _           => println!("Invalid input"),
        }
    }
    let trails: bool;

    loop {
        println!("Do you want to have trails? Yes/No:");
        input.clear();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        let trimmed = input.trim().to_lowercase();

        match trimmed.as_str() {
            "y" | "yes" => { trails = true; break; }
            "n" | "no"  => { trails = false; break; }
            _           => println!("Invalid input"),
        }
    }
    let mut num_important_bodies = 0;
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
        velocity: DVec2::new(0., 0.),
        radius: STAR_RADIUS,
        visible_radius: STAR_VISIBLE_RADIUS,
        color: YELLOW,
        name: String::from("Sun"),
        kinetic_energy: 0.
    };
    star.update_kinetic_energy();
    system[0] = star;
    num_important_bodies += 1;
    let gamma: f64 = 2.0 * std::f64::consts::PI;

    for i in 0..EARTH_NUMBER {
        let angular_position: f64 = gamma * i as f64 / EARTH_NUMBER as f64;
        let earth_x_position: f64 = CENTER_COORDS[0] + angular_position.cos() * EARTH_ORBITAL_RADIUS;
        let earth_y_position: f64 = CENTER_COORDS[1] + angular_position.sin() * EARTH_ORBITAL_RADIUS;
        let earth_position: DVec2 = DVec2::new(earth_x_position, earth_y_position);

        let velocity_direction: f64 = angular_position + 0.5 * std::f64::consts::PI;
        let orbital_speed = EARTH_ORBITAL_VELOCITY;
        let earth_x_velocity: f64 = velocity_direction.cos() * orbital_speed;
        let earth_y_velocity: f64 = velocity_direction.sin() * orbital_speed;

        let earth_velocity: DVec2 = DVec2::new(earth_x_velocity, earth_y_velocity);

        let mut new_planet: Particle = Particle {
            mass: EARTH_MASS*2.,
            position: earth_position,
            velocity: 0.8*earth_velocity,
            radius: EARTH_RADIUS,
            visible_radius: PLANET_VISIBLE_RADIUS,
            color: BLUE,
            name: String::from(format!("Planet {}", i+1)),
            kinetic_energy: 0.
        };
        new_planet.update_kinetic_energy();
        system[i+1] = new_planet;
        num_important_bodies += 1;
    }

    let gamma: f64 = 2.0 * std::f64::consts::PI;

    // Generates a number of comets with varying masses, positions, and velocities
    for i in EARTH_NUMBER+1..NUMBER_OF_BODIES {
        let comet_orbital_radius: f64 = gen_range(COMET_ORBITAL_RADIUS_VARIANCE_MIN, COMET_ORBITAL_RADIUS_VARIANCE_MAX) * COMET_ORBITAL_RADIUS;
        let angular_position: f64 = gamma * gen_range(0.0, 1.0);
        let comet_x_position: f64 = CENTER_COORDS[0] + angular_position.cos() * comet_orbital_radius;
        let comet_y_position: f64 = CENTER_COORDS[1] + angular_position.sin() * comet_orbital_radius;
        let comet_position: DVec2 = DVec2::new(comet_x_position, comet_y_position);

        let velocity_direction: f64 = angular_position + 0.5 * std::f64::consts::PI;
        let orbital_speed = calculate_orbital_speed(&system[0], comet_position);
        let comet_x_velocity: f64 = velocity_direction.cos() * orbital_speed;
        let comet_y_velocity: f64 = velocity_direction.sin() * orbital_speed;

        let comet_velocity: DVec2 = DVec2::new(comet_x_velocity, comet_y_velocity);

        let comet_mass: f64 = gen_range(COMET_MASS_VARIANCE_MIN, COMET_MASS_VARIANCE_MAX) * COMET_MASS;
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

    let mut collision_counter: u32 = 0;

    // This ticker will count the amount of frames multiplied by the number of bodies
    let mut trail_point_counter: usize = 0;

    let mut total_physics_ticks: usize = 0;
    let mut seconds_passed_in_sim: f64 = 0.0;


    // old_positions stores for a decided amount of frames the past the positions of all bodies to draw later
    let mut trail_values = vec![vec![vec![DVec2::new(0., 0.), DVec2::new(0., 0.)]; OLD_FRAME_LIMIT]; num_important_bodies];

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
    while frames_waited < 1*(1./FRAMERATE) as i32 {
        draw_bodies(&system);
        next_frame().await;
        frames_waited += 1;
    }

    loop {
        clear_background(BLACK);



        for _i in 0..TICKS_PER_FRAME {
            total_physics_ticks += 1;
            // Parallel calculation of all the forces acting on the bodies using the calculate_g_force method
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
            collision_counter += collision_engine(&mut system);

            seconds_passed_in_sim += DT;
        }
        if file_write {
            if let Some(ref mut w) = wtr {
                if rows_added < ROW_LIMIT {
                    add_physical_data(&system, seconds_passed_in_sim, w, rows_added);
                    rows_added += 1;
                }
            }
        }



        // Adds new positions to old_position and iterates ticker
        if trails {
            for i in 0..num_important_bodies {
                trail_values[i][trail_point_counter % OLD_FRAME_LIMIT][0] = system[i].position;
                trail_values[i][trail_point_counter % OLD_FRAME_LIMIT][1] = system[i].velocity;
            }
            trail_point_counter += 1;

            let recent_point = trail_point_counter % OLD_FRAME_LIMIT;
            let gap_point = if recent_point == 0 { OLD_FRAME_LIMIT - 1 } else { recent_point - 1 };
            // Draws the trail using old_positions
            if trail_point_counter < OLD_FRAME_LIMIT {
                for i in 0..trail_point_counter {
                    for j in 0..num_important_bodies {
                        draw_circle(scale_window(trail_values[j][i][0][0]),
                                    scale_window(trail_values[j][i][0][1]),
                                    TRAIL_RADIUS,
                                    velocity_to_color(trail_values[j][i][1]));
                    }
                }
            } else {
                for i in 0..num_important_bodies {
                    for j in 0..OLD_FRAME_LIMIT {
                        if j != gap_point {
                            let pos_1 = trail_values[i][j][0];
                            let pos_2 = trail_values[i][(j + 1) % OLD_FRAME_LIMIT][0];
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


        



        // Draws main bodies
        draw_bodies(&system);


        draw_poly_lines(scale_window(CENTER_COORDS[0]),
                        scale_window(CENTER_COORDS[1]),
                        64,
                        scale_window(EARTH_ORBITAL_RADIUS),
                        0.,
                        1.,
                        RED);

        let years_passed_in_sim: String = (seconds_passed_in_sim / SECONDS_IN_YEAR).to_string();
        let mut info_on_screen = format!("Years Passed: {} | Total Physics Ticks: {}",
                        &years_passed_in_sim[0..5],
                        total_physics_ticks,
        );
        draw_text(&info_on_screen, 10.0, (SCREEN_SIZE-80) as f32, 20.0, WHITE);
        if file_write {
            info_on_screen.push_str(&format!(" | Still Writing: {} (with {} rows)",
                            rows_added < ROW_LIMIT,
                            rows_added));

        }
        if collision_counter > 0 {
            info_on_screen.push_str(&format!(" | Collision Count: {}", collision_counter));
        }


        draw_text(&info_on_screen, 10.0, (SCREEN_SIZE-80) as f32, 20.0, WHITE);


        next_frame().await
    }





}
