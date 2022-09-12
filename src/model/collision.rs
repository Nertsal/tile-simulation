use super::*;

impl Model {
    pub fn collide_tiles(&mut self, tile_index: usize, other_index: usize) {
        let (tile_deltas, other_deltas) = self.collide_split_impulses(tile_index, other_index);
        self.apply_deltas(tile_index, tile_deltas);
        self.apply_deltas(other_index, other_deltas);
    }

    /// The first (out of 2) step of calculating tile collisions.
    /// Returns the result of collision for both tiles split into all 4 directions
    /// (clock-wise starting from upwards).
    fn collide_split_impulses(
        &mut self,
        tile_index: usize,
        other_index: usize,
    ) -> ([Coord; 4], [Coord; 4]) {
        if let Some((tile, other)) = self.tiles.get_two(tile_index, other_index) {
            let normal = self.get_normal(tile_index, other_index);
            let delta = solve_tile_impulses(tile, other, normal);
            let tile = self.tiles.get_mut(tile_index).unwrap();
            tile.tick_velocity = remove_component(tile.tick_velocity, normal);
            let other = self.tiles.get_mut(other_index).unwrap();
            other.tick_velocity = remove_component(other.tick_velocity, -normal);
            (add_split(Vec2::ZERO, delta), add_split(Vec2::ZERO, -delta))
        } else {
            ([Coord::ZERO; 4], [Coord::ZERO; 4])
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

    fn apply_deltas(&mut self, tile_index: usize, tile_deltas: [Coord; 4]) {
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
            for (delta, pos) in tile_deltas.iter().zip(pos_deltas) {
                tile.velocity = pos.map(|x| Coord::new(x as f32)) * *delta;
                if let Some(pos) = position.shift(pos, self.get_size().x) {
                    let other_index = pos.to_index(self.get_size().x);
                    let normal = self.get_normal(tile_index, other_index);
                    if let Some(other) = self.tiles.get_mut(other_index) {
                        let delta = solve_tile_impulses(&tile, other, normal);
                        tile_delta += delta;
                        other.velocity -= delta;
                    }
                }
            }
            let tile = self.tiles.get_mut(tile_index).unwrap();
            tile.velocity += tile_delta;
        }
    }
}

/// Adds the delta to the given vector and splits into all 4 directions.
fn add_split(initial: Vec2<Coord>, delta: Vec2<Coord>) -> [Coord; 4] {
    let mut left = Coord::ZERO;
    let mut right = Coord::ZERO;
    let mut down = Coord::ZERO;
    let mut up = Coord::ZERO;

    let coef = Coord::new(0.5);
    if delta.x.abs() >= delta.y.abs() {
        let dy = delta.x.abs() * coef;
        down += dy;
        up += dy;
    } else {
        let dx = delta.y.abs() * coef;
        left += dx;
        right += dx;
    }

    *if initial.x > Coord::ZERO {
        &mut right
    } else {
        &mut left
    } += initial.x.abs();
    *if initial.y > Coord::ZERO {
        &mut up
    } else {
        &mut down
    } += initial.y.abs();

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

fn solve_tile_impulses(tile: &Tile, other: &Tile, normal: Vec2<Coord>) -> Vec2<Coord> {
    if tile.is_static && other.is_static {
        return Vec2::ZERO;
    }
    if other.is_static {
        let bounciness = Coord::new(1.0 + 0.2);
        let tile_projection = normal * Vec2::dot(tile.velocity, normal);
        return -tile_projection * bounciness;
    }
    if tile.is_static {
        let bounciness = Coord::new(1.0 + 0.2);
        let other_projection = normal * Vec2::dot(other.velocity, normal);
        return other_projection * bounciness;
    }
    collide_impulses(tile.velocity, other.velocity, normal)
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

fn remove_component(vec: Vec2<Coord>, dir: Vec2<Coord>) -> Vec2<Coord> {
    let dir = dir.normalize_or_zero();
    let dot = Vec2::dot(vec, dir);
    vec - dir * dot.max(Coord::ZERO)
}

/// Returns the rotation of the tile's edges. This is used to introduce a little bit
/// of 'chaos' in the system when performing collisions.
pub fn get_tile_edge_rotation(position: Position) -> R32 {
    const EDGE_ROTATION: f32 = 0.00;
    let mult = ((position.position.x + position.position.y) % 2) as f32 * 2.0 - 1.0;
    r32(mult * EDGE_ROTATION)
}
