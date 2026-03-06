use crate::constants::*;
use crate::helpers::{Particle, velocity_to_color};
use crate::init_helpers::ConfigValues;
use macroquad::color::Color;
use macroquad::input::is_key_down;
use macroquad::math::{DVec2, Vec2};
use macroquad::prelude::{KeyCode, draw_circle, draw_line};

pub struct ScreenValues {
    pub screen_size_pixels: u32,
    pub screen_size_meters: f64,
    pub offset_pixels: Vec2,
}

impl ScreenValues {
    pub fn physical_pos_to_screen_coords(&self, position: DVec2) -> Vec2 {
        let position_in_pixels: Vec2 = Vec2::new(
            meters_to_pixel(position.x, self),
            meters_to_pixel(position.y, self),
        );
        position_in_pixels + Vec2::new(
            self.screen_size_pixels as f32 / 2.0,
            self.screen_size_pixels as f32 / 2.0,
        ) - self.offset_pixels
    }

    pub fn initialize(&mut self, screen_size_pixels: u32, screen_size_meters: f64) {
        self.screen_size_pixels = screen_size_pixels;
        self.screen_size_meters = screen_size_meters;

        self.offset_pixels = Vec2::ZERO;
    }
    pub fn update(&mut self) {
        let down = is_key_down(KeyCode::S);
        let up = is_key_down(KeyCode::W);
        let left = is_key_down(KeyCode::A);
        let right = is_key_down(KeyCode::D);
        let zoom_in = is_key_down(KeyCode::Q);
        let zoom_out = is_key_down(KeyCode::E);
        let reset = is_key_down(KeyCode::R);
        if reset {
            self.screen_size_meters = AU;
            self.offset_pixels = Vec2::ZERO;
        } else {
            let direction = OFFSET_VELOCITY
                * Vec2::new(
                right as u8 as f32 - left as u8 as f32,
                -(up as u8 as f32) + down as u8 as f32,
            );

            if zoom_in && !zoom_out {
                self.screen_size_meters *= ZOOM_VELOCITY
            } else if zoom_out && !zoom_in {
                self.screen_size_meters /= ZOOM_VELOCITY
            }

            self.offset_pixels += direction;
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
