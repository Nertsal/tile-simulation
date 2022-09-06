use crate::model::*;
use crate::render::Render;

use super::*;

pub struct Game {
    model: Model,
    render: Render,
    framebuffer_size: Vec2<usize>,
    selected_tile: Tile,
}

impl Game {
    pub fn new(geng: &Geng) -> Self {
        Self {
            model: Model::new(),
            render: Render::new(geng),
            framebuffer_size: vec2(1, 1),
            selected_tile: Tile::Empty,
        }
    }

    fn click(&mut self, position: Vec2<f64>) {
        let world_pos = self.render.camera.screen_to_world(
            self.framebuffer_size.map(|x| x as f32),
            position.map(|x| x as f32),
        );
        if let Some(tile_pos) = self.tile_pos(world_pos) {
            self.model.set_tile(tile_pos, self.selected_tile);
        }
    }

    fn tile_pos(&self, world_pos: Vec2<f32>) -> Option<Position> {
        let tile_pos = (world_pos / crate::render::TILE_SIZE).map(|x| x.floor() as i64);
        if tile_pos.iter().any(|x| *x < 0) {
            None
        } else {
            Some(Position {
                position: tile_pos.map(|x| x as usize),
            })
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        self.render.draw_model(&self.model, framebuffer);
        self.render.draw_ui(&self.selected_tile, framebuffer);
    }

    fn fixed_update(&mut self, _delta_time: f64) {
        self.model.tick();
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyDown { key } => match key {
                geng::Key::Num0 => self.selected_tile = Tile::Empty,
                geng::Key::Num1 => self.selected_tile = Tile::Sand,
                _ => {}
            },
            geng::Event::MouseDown {
                position,
                button: geng::MouseButton::Left,
            } => {
                self.click(position);
            }
            _ => {}
        }
    }
}
