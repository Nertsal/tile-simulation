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

    pub fn to_index(self, width: usize) -> usize {
        self.position.x + self.position.y * width
    }

    pub fn shift(self, delta: Vec2<isize>, width: usize) -> Option<Self> {
        let pos = self.position.map(|x| x as isize);
        let pos = pos + delta;
        if pos.iter().any(|x| *x < 0 || *x >= width as isize) {
            return None;
        }
        let position = pos.map(|x| x as usize);
        Some(Self { position })
    }
}
