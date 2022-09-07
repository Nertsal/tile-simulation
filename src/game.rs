use crate::model::*;
use crate::render::Render;

use super::*;

pub struct Game {
    geng: Geng,
    model: Model,
    render: Render,
    framebuffer_size: Vec2<usize>,
    selected_tile: Tile,
    last_mouse_pos: Vec2<f64>,
}

impl Game {
    pub fn new(geng: &Geng) -> Self {
        Self {
            geng: geng.clone(),
            model: Model::new(),
            render: Render::new(geng),
            framebuffer_size: vec2(1, 1),
            selected_tile: Tile::Empty,
            last_mouse_pos: Vec2::ZERO,
        }
    }

    fn tile_pos(&self, world_pos: Vec2<f32>) -> Option<Position> {
        let tile_pos = (world_pos / crate::render::TILE_SIZE).map(|x| x.floor() as i64);
        let size = self.model.get_size();
        if tile_pos.iter().any(|x| *x < 0)
            || tile_pos.x >= size.x as i64
            || tile_pos.y >= size.y as i64
        {
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

    fn update(&mut self, delta_time: f64) {
        if self
            .geng
            .window()
            .is_button_pressed(geng::MouseButton::Left)
        {
            let position = self.geng.window().mouse_pos();
            let world_pos = self.render.camera.screen_to_world(
                self.framebuffer_size.map(|x| x as f32),
                position.map(|x| x as f32),
            );
            if let Some(tile_pos) = self.tile_pos(world_pos) {
                let last_mouse_pos = self.render.camera.screen_to_world(
                    self.framebuffer_size.map(|x| x as f32),
                    self.last_mouse_pos.map(|x| x as f32),
                );
                let velocity =
                    (world_pos - last_mouse_pos) / delta_time as f32 / 20.0 + vec2(0.0, -0.5);
                let velocity = velocity.map(Coord::new);
                self.model.set_tile(tile_pos, velocity, self.selected_tile);
            }
        }

        self.last_mouse_pos = self.geng.window().mouse_pos();
    }

    fn fixed_update(&mut self, _delta_time: f64) {
        self.model.tick();
    }

    fn handle_event(&mut self, event: geng::Event) {
        if let geng::Event::KeyDown { key } = event {
            match key {
                geng::Key::Num0 => self.selected_tile = Tile::Empty,
                geng::Key::Num1 => self.selected_tile = Tile::Sand,
                _ => {}
            }
        }
    }
}
