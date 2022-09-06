use super::*;

use model::*;

pub struct Game {
    model: Model,
}

impl Game {
    pub fn new() -> Self {
        Self {
            model: Model::new(),
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);
    }
}
