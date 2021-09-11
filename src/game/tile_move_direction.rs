use macroquad::prelude::{vec2, IVec2, Vec2};

#[derive(Debug)]
pub enum DirectionError {
    ZeroVector,
    LongVector,
}

impl std::fmt::Display for DirectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroVector => write!(f, "direction must not be a zero vector"),
            Self::LongVector => write!(f, "direction's coordinates must be in range -1..=1"),
        }
    }
}

impl std::error::Error for DirectionError {}

#[derive(Clone, Copy, Debug)]
pub struct TileMoveDirection {
    direction: IVec2,
}

impl TileMoveDirection {
    pub fn new(direction: IVec2) -> Result<Self, DirectionError> {
        // Direction cannot be a zero vector
        if direction == IVec2::ZERO {
            return Err(DirectionError::ZeroVector);
        }

        // Direction cannote be longer than 1 tile (diagonal included)
        if direction.x.abs() > 1 || direction.y.abs() > 1 {
            return Err(DirectionError::LongVector);
        }

        Ok(Self { direction })
    }

    pub fn direction(&self) -> IVec2 {
        self.direction
    }

    pub fn direction_f32(&self) -> Vec2 {
        let direction = self.direction();
        vec2(direction.x as f32, direction.y as f32)
    }
}

impl From<IVec2> for TileMoveDirection {
    fn from(direction: IVec2) -> Self {
        Self::new(direction).unwrap()
    }
}
