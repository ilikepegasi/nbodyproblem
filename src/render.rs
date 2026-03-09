use crate::constants::*;
use crate::helpers::{Particle, velocity_to_color};
use crate::init_helpers::ConfigValues;
use crate::render::LockedTarget::Planet;
use macroquad::color::{Color, RED};
use macroquad::input::{is_key_down, is_key_released};
use macroquad::math::{DVec2, Vec2};
use macroquad::prelude::{KeyCode, draw_circle, draw_line};
use std::any::Any;
use std::string::ToString;

pub enum Mode {
    Free,
    Locked(LockedTarget),
}

pub enum LockedTarget {
    Planet(usize),
}
pub struct ScreenValues {
    pub screen_size_pixels: u32,
    pub screen_size_meters: f64,
    pub center_meters: DVec2,
    pub mode: Mode,
}
impl ScreenValues {
    pub fn physical_pos_to_screen_coords(&self, position: DVec2) -> Vec2 {
        let relative_position = position - self.center_meters;
        let centered_pixel = self.screen_size_pixels / 2;
        centered_pixel as f32
            + Vec2::new(
                meters_to_pixel(relative_position.x, self),
                meters_to_pixel(relative_position.y, self),
            )
    }

    pub fn initialize(&mut self, screen_size_pixels: u32, screen_size_meters: f64) {
        self.screen_size_pixels = screen_size_pixels;
        self.screen_size_meters = screen_size_meters;

        self.center_meters = DVec2::ZERO;
        self.mode = Mode::Free;
    }
    pub fn update_free(&mut self) {
        let down = is_key_down(KeyCode::S);
        let up = is_key_down(KeyCode::W);
        let left = is_key_down(KeyCode::A);
        let right = is_key_down(KeyCode::D);
        let reset = is_key_down(KeyCode::R);
        if reset {
            self.screen_size_meters = AU;
            self.center_meters = DVec2::ZERO;
        } else {
            self.zoom();

            let meters_per_pixel = self.screen_size_meters / self.screen_size_pixels as f64;
            let direction = OFFSET_VELOCITY as f64
                * meters_per_pixel
                * DVec2::new(
                    right as u8 as f64 - left as u8 as f64,
                    -(up as u8 as f64) + down as u8 as f64,
                );

            self.center_meters += direction;
        }
    }

    pub fn zoom(&mut self) {
        let zoom_in = is_key_down(KeyCode::Q);
        let zoom_out = is_key_down(KeyCode::E);
        if zoom_in && !zoom_out {
            self.screen_size_meters *= ZOOM_VELOCITY;
        } else if zoom_out && !zoom_in {
            self.screen_size_meters /= ZOOM_VELOCITY;
        }
    }

    pub fn update_locked(&mut self, system: &Vec<Particle>, to_switch: usize) {
        let reset = is_key_down(KeyCode::R);
        let reset = is_key_down(KeyCode::R);
        if reset {
            self.screen_size_meters = AU;
            self.center_meters = DVec2::ZERO;
        } else {
            self.zoom();
            self.center_meters = system[to_switch].position;
        }
    }
}

pub fn meters_to_pixel(distance: f64, screen_values: &ScreenValues) -> f32 {
    (distance / screen_values.screen_size_meters) as f32 * screen_values.screen_size_pixels as f32
}

pub fn draw_bodies(system: &Vec<Particle>, screen_values: &ScreenValues) {
    for i in 0..system.len() {
        let screen_position = screen_values.physical_pos_to_screen_coords(system[i].position);
        let visible_radius_calculated = system[i].generate_visible_radius();

        draw_circle(
            screen_position.x,
            screen_position.y,
            visible_radius_calculated.max(meters_to_pixel(system[i].radius, screen_values)),
            system[i].color,
        );
    }
}

pub fn draw_trails(
    num_important_bodies: usize,
    system: &Vec<Particle>,
    trail_point_counter: &mut usize,
    trail_values: &mut Vec<Vec<(DVec2, Color)>>,
    log_min_speed: f32,
    log_max_speed: f32,
    init_output: &ConfigValues,
    screen_values: &ScreenValues,
) {
    for i in 0..num_important_bodies {
        trail_values[i][*trail_point_counter % init_output.trail_length].0 = system[i].position;
        trail_values[i][*trail_point_counter % init_output.trail_length].1 =
            velocity_to_color(system[i].velocity, log_min_speed, log_max_speed);
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
                    let screen_pos_1 = screen_values.physical_pos_to_screen_coords(pos_1);
                    let screen_pos_2 = screen_values.physical_pos_to_screen_coords(pos_2);

                    draw_line(
                        screen_pos_1.x,
                        screen_pos_1.y,
                        screen_pos_2.x,
                        screen_pos_2.y,
                        TRAIL_RADIUS,
                        trail_values[i][j].1,
                    );
                }
            }
        }
    }
}

pub fn cross(screen_values: &ScreenValues) {
    draw_line(
        (screen_values.screen_size_pixels / 2) as f32,
        (screen_values.screen_size_pixels / 2) as f32 + 10.,
        (screen_values.screen_size_pixels / 2) as f32,
        (screen_values.screen_size_pixels / 2) as f32 - 10.,
        1.,
        Color::from_rgba(255, 0, 0, 120),
    );

    draw_line(
        (screen_values.screen_size_pixels / 2) as f32 - 10.,
        (screen_values.screen_size_pixels / 2) as f32,
        (screen_values.screen_size_pixels / 2) as f32 + 10.,
        (screen_values.screen_size_pixels / 2) as f32,
        1.,
        Color::from_rgba(255, 0, 0, 120),
    );
}

pub fn render_call(
    trails: bool,
    screen_values: &mut ScreenValues,
    system: &mut Vec<Particle>,
    init_output: &ConfigValues,
    important_bodies_added: usize,
    mut trail_point_counter: &mut usize,
    mut trail_values: &mut Vec<Vec<(DVec2, Color)>>,
) {


    if is_key_released(KeyCode::Right) {
        if let Mode::Locked(Planet(id)) = screen_values.mode {
            screen_values.mode = Mode::Locked(Planet((id + 1) % system.len()));
        }
    }
    if is_key_released(KeyCode::Left) {
        if let Mode::Locked(Planet(id)) = screen_values.mode {
            let new_idx = if id == 0 { system.len() - 1 } else { id - 1 };
            screen_values.mode = Mode::Locked(Planet(new_idx));
        }
    }

    match screen_values.mode {
        Mode::Free => screen_values.update_free(),
        Mode::Locked(Planet(id)) => screen_values.update_locked(system, id),
    }
    // Draws main bodies
    draw_bodies(&system, &screen_values);
    cross(&screen_values);
    if is_key_released(KeyCode::Tab) {
        screen_values.mode = match screen_values.mode {
            Mode::Free => Mode::Locked(Planet(0)),
            Mode::Locked(_) => Mode::Free,
        };
    }
    if trails {
        draw_trails(
            important_bodies_added,
            &system,
            &mut trail_point_counter,
            &mut trail_values,
            init_output.color_vel_range.0 as f32,
            init_output.color_vel_range.1 as f32,
            &init_output,
            &screen_values,
        );
    }

}
