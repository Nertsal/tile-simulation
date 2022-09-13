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

    pub fn draw_model(
        &self,
        model: &Model,
        draw_velocities: game::VelocityVisualMode,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.draw_grid(model, framebuffer);
        self.draw_tiles(model, framebuffer);
        self.draw_velocities(model, draw_velocities, framebuffer);
    }

    pub fn draw_ui(
        &self,
        model: &Model,
        selected_tile: TileType,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.draw_selected_tile(selected_tile, framebuffer);
        let world_pos = self.camera.screen_to_world(
            framebuffer.size().map(|x| x as f32),
            self.geng.window().mouse_pos().map(|x| x as f32),
        );
        if let Some(tile_pos) = game::tile_pos(model, world_pos) {
            if let Some(tile) = model.get_tile(tile_pos) {
                self.draw_tile_info(tile, framebuffer);
            }
        }
    }

    fn draw_tile_info(&self, tile: &Tile, framebuffer: &mut ugli::Framebuffer) {
        if matches!(tile.tile_type, TileType::Empty) || tile.physics.is_static {
            return;
        }
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        let screen = AABB::ZERO.extend_positive(framebuffer_size);
        let pos = vec2(screen.center().x, screen.y_max) + vec2(0.0, -0.1) * screen.height();
        let font = &**self.geng.default_font();
        let font_size = 0.05 * screen.height();
        let text = format!(
            "{:?}: ({:.1}, {:.1}), tick: ({:.1}, {:.1})",
            tile.tile_type,
            tile.velocity.x,
            tile.velocity.y,
            tile.tick_velocity.x,
            tile.tick_velocity.y
        );
        draw_2d::Text::unit(font, text, Color::WHITE)
            .scale_uniform(font_size)
            .translate(pos)
            .draw_2d(&self.geng, framebuffer, &geng::PixelPerfectCamera);
    }

    fn draw_selected_tile(&self, selected_tile: TileType, framebuffer: &mut ugli::Framebuffer) {
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

    fn draw_velocities(
        &self,
        model: &Model,
        mode: game::VelocityVisualMode,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if let game::VelocityVisualMode::Off = mode {
            return;
        }
        for (position, tile) in model.get_tiles() {
            if matches!(tile.tile_type, TileType::Empty) || tile.physics.is_static {
                continue;
            }
            let (velocity, color) = match mode {
                game::VelocityVisualMode::Off => unreachable!(),
                game::VelocityVisualMode::On => (tile.velocity, Color::BLUE),
                game::VelocityVisualMode::Tick => (tile.tick_velocity, Color::GREEN),
            };
            let len = velocity.len();
            if len < Coord::new(0.1) {
                continue;
            }
            let position = position.position.map(|x| x as f32 + 0.5) * TILE_SIZE;
            const ARROW_WIDTH: f32 = 0.1;
            const ARROW_HEAD_SIZE: f32 = 0.1;
            let vertices = vec![
                vec2(0.0, ARROW_WIDTH / 2.0),
                vec2(0.0, -ARROW_WIDTH / 2.0),
                vec2(len.as_f32(), -ARROW_WIDTH / 2.0),
                vec2(len.as_f32(), -ARROW_WIDTH / 2.0 - ARROW_HEAD_SIZE / 2.0),
                vec2(len.as_f32() + ARROW_HEAD_SIZE / 2.0, 0.0),
                vec2(len.as_f32(), ARROW_WIDTH / 2.0 + ARROW_HEAD_SIZE / 2.0),
                vec2(len.as_f32(), ARROW_WIDTH / 2.0),
                vec2(0.0, ARROW_WIDTH / 2.0),
            ];
            let transform = Mat3::translate(position) * Mat3::rotate(tile.velocity.arg().as_f32());
            draw_2d::Polygon::new(vertices, color).draw_2d_transformed(
                &self.geng,
                framebuffer,
                &self.camera,
                transform,
            );
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
        TileType::Barrier => Color::WHITE,
        TileType::Water => Color::rgb(0.0, 0.5, 0.9),
    }
}
