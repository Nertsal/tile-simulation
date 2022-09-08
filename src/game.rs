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
    draw_velocities: bool,
}

impl Game {
    pub fn new(geng: &Geng) -> Self {
        Self {
            geng: geng.clone(),
            model: Model::new(),
            render: Render::new(geng),
            framebuffer_size: vec2(1, 1),
            selected_tile: Tile::empty(),
            last_mouse_pos: Vec2::ZERO,
            draw_velocities: false,
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        self.render
            .draw_model(&self.model, self.draw_velocities, framebuffer);
        self.render
            .draw_ui(&self.model, self.selected_tile.tile_type, framebuffer);
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
            if let Some(tile_pos) = tile_pos(&self.model, world_pos) {
                let last_mouse_pos = self.render.camera.screen_to_world(
                    self.framebuffer_size.map(|x| x as f32),
                    self.last_mouse_pos.map(|x| x as f32),
                );
                let velocity =
                    (world_pos - last_mouse_pos) / delta_time as f32 / 50.0 + vec2(0.0, -1.0);
                let velocity = velocity.map(Coord::new);
                let mut tile = self.selected_tile;
                tile.velocity = velocity;
                self.model.set_tile(tile_pos, tile);
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
                geng::Key::Num0 => self.selected_tile = Tile::empty(),
                geng::Key::Num1 => self.selected_tile = Tile::new_static(TileType::Barrier),
                geng::Key::Num2 => self.selected_tile = Tile::new(TileType::Sand),
                geng::Key::F1 => self.draw_velocities = !self.draw_velocities,
                _ => {}
            }
        }
    }
}

pub fn tile_pos(model: &Model, world_pos: Vec2<f32>) -> Option<Position> {
    let tile_pos = (world_pos / crate::render::TILE_SIZE).map(|x| x.floor() as i64);
    let size = model.get_size();
    if tile_pos.iter().any(|x| *x < 0) || tile_pos.x >= size.x as i64 || tile_pos.y >= size.y as i64
    {
        None
    } else {
        Some(Position {
            position: tile_pos.map(|x| x as usize),
        })
    }
}
