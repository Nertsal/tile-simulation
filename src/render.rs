use geng::{Camera2d, Draw2d};

use crate::model::*;

use super::*;

const GRID_WIDTH: f32 = 0.05;
const GRID_COLOR: Color<f32> = Color::GRAY;
const TILE_SIZE: Vec2<f32> = vec2(1.0, 1.0);

pub struct Render {
    geng: Geng,
    camera: Camera2d,
}

impl Render {
    pub fn new(geng: &Geng) -> Self {
        Self {
            geng: geng.clone(),
            camera: Camera2d {
                center: vec2(5.0, 5.0),
                rotation: 0.0,
                fov: 20.0,
            },
        }
    }

    pub fn draw(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        self.draw_grid(10, 10, framebuffer);
    }

    fn draw_grid(&self, width: usize, height: usize, framebuffer: &mut ugli::Framebuffer) {
        let bounds = AABB::ZERO.extend_positive(vec2(width, height).map(|x| x as f32) * TILE_SIZE);
        // Columns
        for x in 0..=width {
            let x = x as f32 * TILE_SIZE.x;
            draw_2d::Segment::new(
                Segment::new(vec2(x, bounds.y_min), vec2(x, bounds.y_max)),
                GRID_WIDTH,
                GRID_COLOR,
            )
            .draw_2d(&self.geng, framebuffer, &self.camera);
        }
        // Rows
        for y in 0..=height {
            let y = y as f32 * TILE_SIZE.y;
            draw_2d::Segment::new(
                Segment::new(vec2(bounds.x_min, y), vec2(bounds.x_max, y)),
                GRID_WIDTH,
                GRID_COLOR,
            )
            .draw_2d(&self.geng, framebuffer, &self.camera);
        }
    }
}
