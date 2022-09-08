use super::*;

impl Model {
    pub fn tick(&mut self) {
        self.apply_gravity();
        self.move_tiles();
    }

    fn apply_gravity(&mut self) {
        for tile in self.tiles.iter_mut() {
            let gravity_scale = match tile.tile_type {
                TileType::Empty => 0.0,
                TileType::Barrier => 0.0,
                TileType::Sand => 1.0,
            };
            let gravity = (GRAVITY * gravity_scale).map(Coord::new);
            tile.velocity += gravity;
        }
    }

    fn move_tiles(&mut self) {
        // Update tick velocities
        for tile in self.tiles.iter_mut() {
            tile.tick_velocity += tile.velocity;
        }

        // Repeatedly calculate tiles while updates are queued
        let mut update_queue: Vec<usize> = self
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, tile)| !matches!(tile.tile_type, TileType::Empty))
            .map(|(i, _)| i)
            .collect();
        while !update_queue.is_empty() {
            // Calculate and perform movement
            let calculation = self.calculate_movement(&mut update_queue);
            self.perform_movement(calculation.moves_to);
            update_queue.extend(calculation.next_updates);
        }
    }

    fn calculate_movement(&mut self, update_queue: &mut Vec<usize>) -> Calculation {
        let mut calculation = Calculation {
            next_updates: Vec::new(),
            state: DataArray::new(self.get_tiles_count(), TileMoveInfo::Queued),
            moves_to: DataArray::from((0..self.get_tiles_count()).map(Some).collect::<Vec<_>>()),
        };
        while let Some(index) = update_queue.pop() {
            self.calculate_tile_move(index, &mut calculation);
        }
        calculation
    }

    fn perform_movement(&mut self, moves_to: DataArray<Option<usize>>) {
        let mut new_tiles = DataArray::new(self.get_tiles_count(), Tile::empty());
        for (from, to) in moves_to.into_iter().enumerate() {
            if let Some(to) = to {
                *new_tiles.get_mut(to).unwrap() = *self.tiles.get(from).unwrap();
            }
        }
        self.tiles = new_tiles;
    }

    fn calculate_tile_move(&mut self, tile_index: usize, calculation: &mut Calculation) {
        // Check if the tile has already been calculated
        match calculation.state.get(tile_index).unwrap() {
            TileMoveInfo::Queued => {}
            TileMoveInfo::Processing => {
                // This tile has already been started to be calculated.
                // The fact that the calculation has returned to that tile
                // indicates that the tile tried to move into another tile
                // that attempted to move into the original tile
                // (or a longer cycle has occured).
                // The solution to this infinite recursion is to perform collisions.
                *calculation.state.get_mut(tile_index).unwrap() = TileMoveInfo::Static;
                return;
            }
            _ => return,
        }

        let tile = self.tiles.get(tile_index).unwrap();

        // Check if the tile is an Empty tile
        if let TileType::Empty = tile.tile_type {
            *calculation.state.get_mut(tile_index).unwrap() = TileMoveInfo::Freed;
            *calculation.moves_to.get_mut(tile_index).unwrap() = None;
            return;
        }

        // Check if the tile is static
        if tile.is_static {
            *calculation.state.get_mut(tile_index).unwrap() = TileMoveInfo::Static;
            return;
        }

        // Indicate that the tile's processing has begun
        *calculation.state.get_mut(tile_index).unwrap() = TileMoveInfo::Processing;

        // Get the movement direction based on the tile's velocity
        let direction = velocity_direction(self.tiles.get(tile_index).unwrap().tick_velocity);
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
                        *calculation.moves_to.get_mut(tile_index).unwrap() = Some(target_index);
                        self.tiles.get_mut(tile_index).unwrap().tick_velocity -=
                            direction.map(|x| Coord::new(x as f32));

                        // Queue update for the moved tile
                        calculation.next_updates.push(target_index);
                        return;
                    } else {
                        // The target tile is occupied either by the target tile itself or by another tile
                        // that will replace the target tile.
                        // Hence, we need to perform collisions
                        self.collide_tiles(tile_index, target_index);
                        *calculation.state.get_mut(tile_index).unwrap() = TileMoveInfo::Static;
                        return;
                    }
                }
                ShiftedPosition::OutOfBounds => {
                    // The tile wants to move out of bounds, but that is impossible.
                    // We need to subtract velocity of the tile in that out of bounds direction.
                    let position = Position::from_index(tile_index, self.get_size().x);
                    let edge_rotation = get_tile_edge_rotation(position);
                    let tile = self.tiles.get_mut(tile_index).unwrap();
                    let direction = direction
                        .map(|x| Coord::new(x as f32))
                        .rotate(edge_rotation);
                    tile.velocity -= direction * Vec2::dot(tile.velocity, direction);
                    tile.tick_velocity -= direction * Vec2::dot(tile.tick_velocity, direction);
                }
            }
        }

        // The tile either has no velocity or has been calculated to stay in place
        *calculation.state.get_mut(tile_index).unwrap() = TileMoveInfo::Static;
    }

    /// Calculates tile collision and changes their velocities and tick_velocities.
    /// The tiles are not checked for adjacency, so collision checks should be done by the caller,
    /// and the difference of their positions is not normalized (which may result in weird
    /// behaviour if not accounted for by the caller).
    fn collide_tiles(&mut self, tile_index: usize, other_index: usize) {
        let tile = match self.tiles.get(tile_index) {
            None => return,
            Some(tile) => tile,
        };
        let other = match self.tiles.get(other_index) {
            None => return,
            Some(tile) => tile,
        };

        let tile_position = Position::from_index(tile_index, self.get_size().x);
        let other_position = Position::from_index(other_index, self.get_size().x);

        let edge_rotation = get_tile_edge_rotation(other_position);
        let normal = (other_position.position.map(|x| Coord::new(x as f32))
            - tile_position.position.map(|x| Coord::new(x as f32)))
        .rotate(edge_rotation);

        let tile_projection = normal * Vec2::dot(tile.velocity, normal);
        let tile_tick_projection = normal * Vec2::dot(tile.tick_velocity, normal);

        let other_projection = normal * Vec2::dot(other.velocity, normal);
        let other_tick_projection = normal * Vec2::dot(other.tick_velocity, normal);

        let relative_velocity = other.velocity - tile.velocity;
        let relative_tick_velocity = other.tick_velocity - tile.tick_velocity;

        let relative_projection = normal * Vec2::dot(relative_velocity, normal);
        let relative_tick_projection = normal * Vec2::dot(relative_tick_velocity, normal);

        // Check for static tiles
        if other.is_static {
            if tile.is_static {
                return;
            }
            let bounciness = Coord::new(1.0 + 0.1);
            let tile = self.tiles.get_mut(tile_index).unwrap();
            tile.velocity -= tile_projection * bounciness;
            tile.tick_velocity -= tile_tick_projection * bounciness;
            return;
        }
        if tile.is_static {
            if other.is_static {
                return;
            }
            let bounciness = Coord::new(1.0 + 0.1);
            let tile = self.tiles.get_mut(other_index).unwrap();
            tile.velocity -= other_projection * bounciness;
            tile.tick_velocity -= other_tick_projection * bounciness;
            return;
        }

        // Both tiles are not static
        let elasticity = Coord::new(1.0);
        let energy_loss = Coord::new(1.0 - 0.1);
        let relative_projection = relative_projection * elasticity;
        let relative_tick_projection = relative_tick_projection * elasticity;
        let tile = self.tiles.get_mut(tile_index).unwrap();
        tile.velocity = (tile.velocity + relative_projection) * energy_loss;
        tile.tick_velocity = (tile.tick_velocity + relative_tick_projection) * energy_loss;
        let other = self.tiles.get_mut(other_index).unwrap();
        other.velocity = (other.velocity - relative_projection) * energy_loss;
        other.tick_velocity = (other.tick_velocity - relative_tick_projection) * energy_loss;
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
    next_updates: Vec<usize>,
    /// Results of tile movement calculations.
    state: DataArray<TileMoveInfo>,
    /// The calculated tile moves.
    moves_to: DataArray<Option<usize>>,
}

/// Transforms normal velocity into a single tile long direction (one of 5 options).
fn velocity_direction(velocity: Vec2<Coord>) -> Vec2<isize> {
    let vel = velocity.map(|x| x.as_f32() as isize);
    match vel.x.abs().cmp(&vel.y.abs()) {
        std::cmp::Ordering::Less => vec2(0, vel.y.signum()),
        _ => vec2(vel.x.signum(), 0),
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

/// Returns the rotation of the tile's edges. This is used to introduce a little bit
/// of 'chaos' in the system when performing collisions.
fn get_tile_edge_rotation(position: Position) -> R32 {
    const EDGE_ROTATION: f32 = 0.05;

    let mult = ((position.position.x + position.position.y) % 2) as f32 * 2.0 - 1.0;
    r32(mult * EDGE_ROTATION)
}
