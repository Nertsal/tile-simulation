use super::*;

impl Model {
    pub fn collide_tiles(&mut self, tile_index: usize, other_index: usize) {
        let (tile_deltas, other_deltas, tile_tick_deltas, other_tick_deltas) =
            self.collide_split_impulses(tile_index, other_index);
        self.apply_deltas(tile_index, tile_deltas, tile_tick_deltas);
        self.apply_deltas(other_index, other_deltas, other_tick_deltas);
    }

    /// The first (out of 2) step of calculating tile collisions.
    /// Returns the result of collision for both tiles split into all 4 directions
    /// (clock-wise starting from upwards).
    fn collide_split_impulses(
        &mut self,
        tile_index: usize,
        other_index: usize,
    ) -> ([Coord; 4], [Coord; 4], [Coord; 4], [Coord; 4]) {
        if let Some((tile, other)) = self.tiles.get_two(tile_index, other_index) {
            let normal = self.get_normal(tile_index, other_index);
            let delta = solve_tile_impulses(
                tile.velocity,
                tile.physics,
                other.velocity,
                other.physics,
                normal,
            );
            let tick_delta = solve_tile_impulses(
                tile.tick_velocity,
                tile.physics,
                other.tick_velocity,
                other.physics,
                normal,
            );
            (
                split_impulse(delta),
                split_impulse(-delta),
                split_impulse(tick_delta),
                split_impulse(-tick_delta),
            )
        } else {
            (
                [Coord::ZERO; 4],
                [Coord::ZERO; 4],
                [Coord::ZERO; 4],
                [Coord::ZERO; 4],
            )
        }
    }

    /// Calculates the normal direction from one tile to another.
    fn get_normal(&self, tile_index: usize, other_index: usize) -> Vec2<Coord> {
        let tile_position = Position::from_index(tile_index, self.get_size().x);
        let other_position = Position::from_index(other_index, self.get_size().x);
        let edge_rotation = get_tile_edge_rotation(other_position);
        (other_position.position.map(|x| Coord::new(x as f32))
            - tile_position.position.map(|x| Coord::new(x as f32)))
        .rotate(edge_rotation)
    }

    fn apply_deltas(
        &mut self,
        tile_index: usize,
        tile_deltas: [Coord; 4],
        tile_tick_deltas: [Coord; 4],
    ) {
        if let Some(tile) = self.tiles.get(tile_index) {
            let mut tile = *tile;
            let position = Position::from_index(tile_index, self.get_size().x);
            let pos_deltas = [(0, 1), (1, 0), (0, -1), (-1, 0)].map(|(x, y)| vec2(x, y));
            let mut tile_delta = tile_deltas
                .iter()
                .zip(pos_deltas)
                .fold(Vec2::ZERO, |acc, (len, dir)| {
                    acc + dir.map(|x| Coord::new(x as f32)) * *len
                });
            let mut tile_tick_delta = tile_tick_deltas
                .iter()
                .zip(pos_deltas)
                .fold(Vec2::ZERO, |acc, (len, dir)| {
                    acc + dir.map(|x| Coord::new(x as f32)) * *len
                });
            for ((delta, tick_delta), pos) in
                tile_deltas.iter().zip(tile_tick_deltas).zip(pos_deltas)
            {
                tile.velocity = pos.map(|x| Coord::new(x as f32)) * *delta;
                tile.tick_velocity = pos.map(|x| Coord::new(x as f32)) * tick_delta;
                if let Some(pos) = position.shift(pos, self.get_size().x) {
                    let other_index = pos.to_index(self.get_size().x);
                    let normal = self.get_normal(tile_index, other_index);
                    if let Some(other) = self.tiles.get_mut(other_index) {
                        let delta = solve_tile_impulses(
                            tile.velocity,
                            tile.physics,
                            other.velocity,
                            other.physics,
                            normal,
                        );
                        tile_delta += delta;
                        other.velocity -= delta;

                        let tick_delta = solve_tile_impulses(
                            tile.tick_velocity,
                            tile.physics,
                            other.tick_velocity,
                            other.physics,
                            normal,
                        );
                        tile_tick_delta += tick_delta;
                        other.tick_velocity -= tick_delta;
                    }
                }
            }
            let tile = self.tiles.get_mut(tile_index).unwrap();
            tile.velocity += tile_delta;
            tile.tick_velocity += tile_tick_delta;
        }
    }
}

/// Splits the given impulse into all 4 directions.
fn split_impulse(delta: Vec2<Coord>) -> [Coord; 4] {
    let mut left = Coord::ZERO;
    let mut right = Coord::ZERO;
    let mut down = Coord::ZERO;
    let mut up = Coord::ZERO;

    let coef = Coord::new(1.0);
    if delta.x.abs() >= delta.y.abs() {
        let dy = delta.x.abs() * coef;
        down += dy;
        up += dy;
    } else {
        let dx = delta.y.abs() * coef;
        left += dx;
        right += dx;
    }

    *if delta.x > Coord::ZERO {
        &mut right
    } else {
        &mut left
    } += delta.x.abs();
    *if delta.y > Coord::ZERO {
        &mut up
    } else {
        &mut down
    } += delta.y.abs();

    [up, right, down, left]
}

fn solve_tile_impulses(
    tile_velocity: Vec2<Coord>,
    tile_physics: TilePhysics,
    other_velocity: Vec2<Coord>,
    other_physics: TilePhysics,
    normal: Vec2<Coord>,
) -> Vec2<Coord> {
    if tile_physics.is_static && other_physics.is_static {
        return Vec2::ZERO;
    }
    if other_physics.is_static {
        let bounciness = Coord::ONE + tile_physics.bounciness;
        let tile_projection = normal * Vec2::dot(tile_velocity, normal);
        return -tile_projection * bounciness;
    }
    if tile_physics.is_static {
        let bounciness = Coord::ONE + other_physics.bounciness;
        let other_projection = normal * Vec2::dot(other_velocity, normal);
        return other_projection * bounciness;
    }
    collide_impulses(tile_velocity, other_velocity, normal)
}

/// Given the impulses of two collided bodies and the collision normal,
/// calculates the delta that should be added to the first impulse,
/// and subtracted from the second one.
fn collide_impulses(a: Vec2<Coord>, b: Vec2<Coord>, normal: Vec2<Coord>) -> Vec2<Coord> {
    let normal = normal.normalize_or_zero();
    let relative = b - a;
    let dot = Vec2::dot(relative, normal);
    // If the projection is positive, then there is basically no collision
    // since the impulses point away from each other
    normal * dot.min(Coord::ZERO) * Coord::new(0.5)
}

/// Returns the rotation of the tile's edges. This is used to introduce a little bit
/// of 'chaos' in the system when performing collisions.
pub fn get_tile_edge_rotation(position: Position) -> R32 {
    const EDGE_ROTATION: f32 = 0.00;
    let mult = ((position.position.x + position.position.y) % 2) as f32 * 2.0 - 1.0;
    r32(mult * EDGE_ROTATION)
}
