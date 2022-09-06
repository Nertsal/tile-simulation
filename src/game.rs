use crate::model::*;
use crate::render::Render;

use super::*;

pub struct Game {
    model: Model,
    render: Render,
}

impl Game {
    pub fn new(geng: &Geng) -> Self {
        Self {
            model: Model::new(),
            render: Render::new(geng),
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);
        self.render.draw(&self.model, framebuffer);
    }
}
