use super::*;

impl Model {
    pub fn tick(&mut self) {
        self.apply_gravity();
        self.move_tiles();
    }

    fn apply_gravity(&mut self) {
        for (tile, velocity) in self.tiles.iter().zip(self.velocities.iter_mut()) {
            let gravity_scale = match tile {
                Tile::Empty => 0.0,
                Tile::Sand => 1.0,
            };
            let gravity = (GRAVITY * gravity_scale).map(Coord::new);
            *velocity += gravity;
        }
    }

    fn move_tiles(&mut self) {
        // Update tick velocities
        for (index, velocity) in self.velocities.iter().enumerate() {
            *self
                .tick_velocities
                .get_mut(index)
                .expect("`tick_velocities` or `velocities` is invalid") += *velocity;
        }

        // Calculate and perform movement
        let movement = self.calculate_movement();
        self.perform_movement(movement);
    }

    fn calculate_movement(&mut self) -> DataArray<Tile> {
        let mut calculation = Calculation {
            queued: (0..self.get_tiles_count()).collect(),
            state: DataArray::new(self.get_tiles_count(), TileMoveInfo::Queued),
            moves: self.tiles.clone(),
        };
        while let Some(index) = calculation.queued.pop() {
            self.calculate_tile_move(index, &mut calculation);
        }
        calculation.moves
    }

    fn perform_movement(&mut self, moves: DataArray<Tile>) {
        self.tiles = moves;
    }

    fn calculate_tile_move(&mut self, tile_index: usize, calculation: &mut Calculation) {
        // Check if the tile has already been calculated
        match calculation.state.get(tile_index).unwrap() {
            TileMoveInfo::Queued => {}
            TileMoveInfo::Processing => {
                // This tile has already been started to be calculated.
                // The fact that the calculation has returned to that tile,
                // indicates that the tile tried to move into another tile
                // that attempted to move into the original tile
                // (or a longer cycle has occured).
                // The solution to this infinite recursion is to perform collisions.
                // TODO: collisions
                *calculation.state.get_mut(tile_index).unwrap() = TileMoveInfo::Static;
                return;
            }
            _ => return,
        }

        // Check if the tile is an Empty tile
        if let Tile::Empty = self.tiles.get(tile_index).unwrap() {
            *calculation.state.get_mut(tile_index).unwrap() = TileMoveInfo::Freed;
            return;
        }

        // Indicate that the tile's processing has begun
        *calculation.state.get_mut(tile_index).unwrap() = TileMoveInfo::Processing;

        // Get the movement direction based on the tile's velocity
        let direction = velocity_direction(*self.velocities.get(tile_index).unwrap());
        if direction != Vec2::ZERO {
            // Try moving tile in the direction
            match self.shift_position(tile_index, direction) {
                ShiftedPosition::Valid(position) => {
                    // The target position in valid, so we need to check for collisions.
                    let target_index = position.to_index(self.get_size().x);
                    // Calculate the target's move
                    // TODO: check for infinite recursions
                    self.calculate_tile_move(target_index, calculation);
                    // Check if target tile's move is Freed
                    let state = calculation.state.get_mut(target_index).unwrap();
                    if let TileMoveInfo::Freed = state {
                        // The target tile will move and can be replaced by the current tile
                        *state = TileMoveInfo::Replaced;
                        *calculation.state.get_mut(tile_index).unwrap() = TileMoveInfo::Freed;
                        *calculation.moves.get_mut(tile_index).unwrap() = Tile::Empty;
                        *calculation.moves.get_mut(target_index).unwrap() =
                            *self.tiles.get(tile_index).unwrap();
                        *self.tick_velocities.get_mut(tile_index).unwrap() -=
                            direction.map(|x| Coord::new(x as f32));
                        return;
                    } else {
                        // The target tile is occupied either by the target tile itself or by another tile
                        // that will replace the target tile.
                        // Hence, we need to perform collisions
                        // TODO: collisions
                        *self.velocities.get_mut(tile_index).unwrap() = Vec2::ZERO;
                        *self.tick_velocities.get_mut(tile_index).unwrap() = Vec2::ZERO;
                    }
                }
                ShiftedPosition::OutOfBounds => {
                    // The tile wants to move out of bounds, but that is impossible.
                    // We need to subtract velocity of the tile in that out of bounds direction.
                    let velocity = self.velocities.get_mut(tile_index).unwrap();
                    *velocity -= *velocity * direction.map(|x| Coord::new(x.abs() as f32));
                    let velocity = self.tick_velocities.get_mut(tile_index).unwrap();
                    *velocity -= *velocity * direction.map(|x| Coord::new(x.abs() as f32));
                }
            }
        }

        // The tile either has no velocity or has been calculated to stay in place
        *calculation.state.get_mut(tile_index).unwrap() = TileMoveInfo::Static;
    }

    fn shift_position(&self, index: usize, direction: Vec2<isize>) -> ShiftedPosition {
        let position = Position::from_index(index, WIDTH)
            .position
            .map(|x| x as isize)
            + direction;
        // TODO: properly check bounds
        if position.iter().any(|x| *x < 0)
            || position.x >= self.get_size().x as isize
            || position.y >= self.get_size().y as isize
        {
            ShiftedPosition::OutOfBounds
        } else {
            ShiftedPosition::Valid(Position {
                position: position.map(|x| x as usize),
            })
        }
    }
}

/// State of calculating tile's movement.
#[derive(Debug, Clone, Copy)]
enum TileMoveInfo {
    /// The tile has not yet been calculated.
    Queued,
    /// The tile's calculation has been initiated, but not yet completed.
    Processing,
    /// The tile will stay in place.
    Static,
    /// The tile will move and its place will be free after the move.
    Freed,
    /// The tile will move but it will be replaced by another tile.
    Replaced,
}

/// A temporary structure to hold intermidate calculation information.
struct Calculation {
    /// Queued tile updates.
    queued: Vec<usize>,
    /// Results of tile movement calculations.
    state: DataArray<TileMoveInfo>,
    /// The calculated tile moves.
    moves: DataArray<Tile>,
}

/// Transforms normal velocity into a single tile long direction (one of 5 options).
fn velocity_direction(velocity: Vec2<Coord>) -> Vec2<isize> {
    let vel = velocity.map(|x| (x.as_f32() as isize).signum());
    match vel.x.abs().cmp(&vel.y.abs()) {
        std::cmp::Ordering::Less => vec2(0, vel.y),
        _ => vec2(vel.x, 0),
    }
}

/// The result of shifting a position in some direction.
#[derive(Debug)]
enum ShiftedPosition {
    /// The position is valid.
    Valid(Position),
    /// The position is out of bounds of any known valid position.
    OutOfBounds,
}
