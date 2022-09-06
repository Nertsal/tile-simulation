use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    // TODO: chunk position
    pub position: Vec2<usize>,
}

impl Position {
    pub fn from_index(index: usize, width: usize) -> Self {
        Self {
            position: vec2(index % width, index / width),
        }
    }

    pub fn index(self, width: usize) -> usize {
        self.position.x + self.position.y * width
    }
}
