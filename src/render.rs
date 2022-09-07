use geng::{Camera2d, Draw2d};

use crate::model::*;

use super::*;

const GRID_WIDTH: f32 = 0.07;
const GRID_COLOR: Color<f32> = Color::GRAY;
pub const TILE_SIZE: Vec2<f32> = vec2(1.0, 1.0);

pub struct Render {
    geng: Geng,
    pub camera: Camera2d,
}

impl Render {
    pub fn new(geng: &Geng) -> Self {
        Self {
            geng: geng.clone(),
            camera: Camera2d {
                center: vec2(15.0, 15.0),
                rotation: 0.0,
                fov: 40.0,
            },
        }
    }

    pub fn draw_model(&self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        self.draw_tiles(model, framebuffer);
        self.draw_grid(model, framebuffer);
    }

    pub fn draw_ui(&self, selected_tile: TileType, framebuffer: &mut ugli::Framebuffer) {
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        let screen = AABB::ZERO.extend_positive(framebuffer_size);
        let aabb = AABB::point(screen.bottom_left() + vec2(0.1, 0.1) * screen.size())
            .extend_positive(vec2(0.1, 0.1) * screen.height());
        let color = tile_color(selected_tile);
        draw_2d::Quad::new(aabb.extend_uniform(0.05 * aabb.height()), Color::GRAY).draw_2d(
            &self.geng,
            framebuffer,
            &geng::PixelPerfectCamera,
        );
        draw_2d::Quad::new(aabb, color).draw_2d(&self.geng, framebuffer, &geng::PixelPerfectCamera);
    }

    fn draw_tiles(&self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        for (position, tile) in model.get_tiles() {
            let position = position.position.map(|x| x as f32) * TILE_SIZE;
            let aabb = AABB::point(position).extend_positive(TILE_SIZE);
            let color = tile_color(tile.tile_type);
            draw_2d::Quad::new(aabb, color).draw_2d(&self.geng, framebuffer, &self.camera);
        }
    }

    fn draw_grid(&self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        let size = model.get_size();
        let bounds = AABB::ZERO.extend_positive(size.map(|x| x as f32) * TILE_SIZE);
        // Columns
        for x in 0..=size.x {
            let x = x as f32 * TILE_SIZE.x;
            draw_2d::Segment::new(
                Segment::new(vec2(x, bounds.y_min), vec2(x, bounds.y_max)),
                GRID_WIDTH,
                GRID_COLOR,
            )
            .draw_2d(&self.geng, framebuffer, &self.camera);
        }
        // Rows
        for y in 0..=size.y {
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

fn tile_color(tile: TileType) -> Color<f32> {
    match tile {
        TileType::Empty => Color::TRANSPARENT_BLACK,
        TileType::Sand => Color::YELLOW,
    }
}
