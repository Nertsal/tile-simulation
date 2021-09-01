use std::{collections::HashMap, sync::Mutex};

use macroquad::prelude::{ivec2, uvec2, IVec2, UVec2};

use crate::constants::{CHUNK_SIZE, CHUNK_SIZE_X, CHUNK_SIZE_Y};

use super::{
    calculator::Calculator,
    tile::{Tile, TileInfo},
};

pub type DataArray<T> = Vec<T>;

pub fn data_array<T: Copy>(default_value: T) -> DataArray<T> {
    let mut data_array = Vec::with_capacity(CHUNK_SIZE);
    for _ in 0..CHUNK_SIZE {
        data_array.push(default_value);
    }
    data_array
}

pub fn default_data_array<T: Default>() -> DataArray<T> {
    let mut data_array = Vec::with_capacity(CHUNK_SIZE);
    for _ in 0..CHUNK_SIZE {
        data_array.push(T::default());
    }
    data_array
}

pub fn tile_index_to_position(tile_index: usize) -> IVec2 {
    let y = tile_index / CHUNK_SIZE_X;
    assert!(y < CHUNK_SIZE_Y);
    ivec2(tile_index as i32 % CHUNK_SIZE_X as i32, y as i32)
}

pub fn tile_position_to_index(tile_position: UVec2) -> usize {
    assert!(
        tile_position.x < CHUNK_SIZE_X as u32 && tile_position.y < CHUNK_SIZE_Y as u32,
        "position {} out of chunk bounds",
        tile_position
    );
    let index = tile_position.x as usize + tile_position.y as usize * CHUNK_SIZE_X;
    index
}

pub struct Chunk {
    pub chunk_pos: IVec2,
    pub tiles: DataArray<bool>,
    pub tile_info: DataArray<Option<TileInfo>>,
    pub need_update: DataArray<bool>,
}

impl Chunk {
    pub fn empty(chunk_pos: IVec2) -> Self {
        Self {
            chunk_pos,
            tiles: data_array(false),
            tile_info: default_data_array(),
            need_update: data_array(false),
        }
    }

    pub fn tiles(&self) -> impl Iterator<Item = (usize, &Option<TileInfo>)> {
        self.tile_info.iter().enumerate()
    }

    pub fn set_tile(&mut self, index: usize, tile_info: Option<TileInfo>) {
        self.need_update[index] = tile_info.is_some();
        self.tile_info[index] = tile_info;
        self.tiles[index] = true;
    }

    pub fn tick(&mut self, calculator: &Mutex<Calculator>) -> DataArray<Option<Option<TileInfo>>> {
        let calculation = self.calculate(calculator);
        self.movement(calculation.moves_to);
        calculation.view_update
    }

    fn calculate(&mut self, calculator: &Mutex<Calculator>) -> ChunkCalculation {
        let mut calculation = ChunkCalculation {
            checked: data_array(false),
            moves: data_array(None),
            moves_to: default_data_array(),
            update_tiles: {
                let mut update_tiles = Vec::new();
                for index in 0..self.need_update.len() {
                    if self.need_update[index] {
                        if self.tiles[index] {
                            update_tiles.push(index);
                        } else {
                            self.need_update[index] = false;
                        }
                    }
                }
                update_tiles
            },
            unknown: data_array(false),
            view_update: default_data_array(),
        };

        let mut dependencies = HashMap::new();
        let mut is_done = false;
        // Calculate all tiles in the chunk
        while !is_done {
            // Clear unknowns
            for unknown_tile in
                calculation
                    .unknown
                    .iter_mut()
                    .enumerate()
                    .filter_map(|(index, unknown_tile)| {
                        if *unknown_tile {
                            *unknown_tile = false;
                            Some(index)
                        } else {
                            None
                        }
                    })
            {
                calculation.update_tiles.push(unknown_tile);
                calculation.checked[unknown_tile] = false;
            }

            // Calculate tiles
            let mut chunk_updates = data_array(None);
            let mut extra_updates = Vec::new();
            let mut cross_moves = HashMap::new();
            while !calculation.update_tiles.is_empty() {
                let update_index = calculation.update_tiles.remove(0);
                let move_info = self.calculate_tile(
                    update_index,
                    &mut calculation,
                    &mut extra_updates,
                    &mut cross_moves,
                    &mut dependencies,
                );
                chunk_updates[update_index] = Some(move_info);
            }

            // Update knowledge about other chunks
            let mut calculator = calculator.lock().unwrap();
            let (updates, cross_moves) = calculator.update(
                self.chunk_pos,
                chunk_updates,
                extra_updates,
                cross_moves,
                &mut dependencies,
            );

            // Register extra updates
            if let Some(updates) = updates {
                for update_index in updates
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, update)| if update { Some(index) } else { None })
                {
                    calculation.update_tiles.push(update_index);
                }
            }

            // Register cross-chunk moves
            if let Some(cross_moves) = cross_moves {
                for (move_to, tile_info) in cross_moves
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, tile_info)| tile_info.map(|tile_info| (index, tile_info)))
                {
                    calculation.view_update[move_to] = Some(Some(tile_info.clone()));
                    calculation.moves_to[move_to] = Some(tile_info);
                    self.need_update[move_to] = true;
                }
            }
            is_done = calculator.is_done();
        }

        calculation
    }

    fn calculate_tile(
        &mut self,
        update_index: usize,
        calculation: &mut ChunkCalculation,
        extra_updates: &mut Vec<Tile>,
        cross_moves: &mut HashMap<Tile, TileInfo>,
        dependencies: &mut HashMap<Tile, MoveInfo>,
    ) -> MoveInfo {
        // If there is no tile
        // or we've calculated that this tile can move,
        // then movement is allowed
        if !self.tiles[update_index] || calculation.moves[update_index].is_some() {
            return MoveInfo::Possible;
        }

        // if this tile's behaviour is unknown,
        // then movement is unknown
        if calculation.unknown[update_index] {
            return MoveInfo::Unknown;
        }

        // If we've alredy checked this tile (implying it can't move)
        // or another tile is going to move here,
        // then movement is not allowed
        let checked = calculation.checked[update_index];
        calculation.checked[update_index] = true;
        if checked || calculation.moves_to[update_index].is_some() {
            return MoveInfo::Impossible;
        }

        // Check for possible moves
        let directions = vec![ivec2(0, -1), ivec2(-1, -1), ivec2(1, -1)];
        for direction in directions {
            // Check if target is inside the current chunk
            match self.shift_position(update_index, direction) {
                Ok(target_index) => {
                    // Inside the current chunk -> check if movement is possible
                    match self.calculate_tile(
                        target_index,
                        calculation,
                        extra_updates,
                        cross_moves,
                        dependencies,
                    ) {
                        MoveInfo::Impossible => (),
                        MoveInfo::Possible => {
                            // Register the move
                            let tile_info =
                                std::mem::take(self.tile_info.get_mut(update_index).unwrap())
                                    .unwrap();
                            calculation.moves[update_index] = Some(target_index);
                            calculation.moves_to[target_index] = Some(tile_info.clone());

                            // Update view
                            calculation.view_update[target_index] = Some(Some(tile_info));
                            if calculation.moves_to[update_index].is_none() {
                                calculation.view_update[update_index] = Some(None);
                            }

                            // Queue update for the next frame
                            self.need_update[target_index] = true;

                            // Update nearby lazy tiles
                            self.update_tiles_around(update_index, 1, calculation, extra_updates);
                            return MoveInfo::Possible;
                        }
                        MoveInfo::Unknown => {
                            calculation.unknown[update_index] = true;
                            return MoveInfo::Unknown;
                        }
                    }
                }
                Err(tile) => {
                    // Outside of the current chunk -> check if it's been calculated, else queue calculation
                    match *dependencies.entry(tile).or_insert(MoveInfo::Unknown) {
                        MoveInfo::Unknown => {
                            calculation.unknown[update_index] = true;
                            return MoveInfo::Unknown;
                        }
                        MoveInfo::Impossible => {}
                        MoveInfo::Possible => {
                            // Register the move
                            let tile_info =
                                std::mem::take(self.tile_info.get_mut(update_index).unwrap())
                                    .unwrap();
                            cross_moves.insert(tile, tile_info);

                            // Update view
                            if calculation.moves_to[update_index].is_none() {
                                calculation.view_update[update_index] = Some(None);
                            }

                            // Update nearby lazy tiles
                            self.update_tiles_around(update_index, 1, calculation, extra_updates);
                            return MoveInfo::Possible;
                        }
                    }
                }
            }
        }

        // There are no possible moves
        // Set this tile into lazy mode
        self.need_update[update_index] = false;
        MoveInfo::Impossible
    }

    fn update_tiles_around(
        &self,
        index: usize,
        distance: i32,
        calculation: &mut ChunkCalculation,
        extra_updates: &mut Vec<Tile>,
    ) {
        // Update tiles in a square around a given tile
        for dx in -distance..=distance {
            for dy in -distance..=distance {
                let shift = ivec2(dx, dy);
                match self.shift_position(index, shift) {
                    Ok(index) => {
                        // Tile is inside the chunk -> queue update
                        if self.tiles[index]
                            && !self.need_update[index]
                            && !calculation.checked[index]
                        {
                            calculation.update_tiles.push(index);
                        }
                    }
                    Err(tile) => {
                        // Tile is outside the chunk -> queue global update
                        extra_updates.push(tile);
                    }
                }
            }
        }
    }

    pub fn shift_position(&self, tile_index: usize, shift: IVec2) -> Result<usize, Tile> {
        // Translate tile index into a vector
        let position = tile_index_to_position(tile_index) + shift;

        // Check if new position is outside the chunk
        let (chunk_shift_x, tile_pos_x) = if position.x < 0 {
            (-1, (position.x + CHUNK_SIZE_X as i32) as u32)
        } else if position.x >= CHUNK_SIZE_X as i32 {
            (1, (position.x - CHUNK_SIZE_X as i32) as u32)
        } else {
            (0, position.x as u32)
        };

        let (chunk_shift_y, tile_pos_y) = if position.y < 0 {
            (-1, (position.y + CHUNK_SIZE_Y as i32) as u32)
        } else if position.y >= CHUNK_SIZE_Y as i32 {
            (1, (position.y - CHUNK_SIZE_Y as i32) as u32)
        } else {
            (0, position.y as u32)
        };

        let tile_position = uvec2(tile_pos_x, tile_pos_y);
        let index = tile_position_to_index(tile_position);

        if chunk_shift_x == 0 && chunk_shift_y == 0 {
            // Inside the chunk
            Ok(index)
        } else {
            // Outside the chunk
            Err(Tile {
                chunk_pos: self.chunk_pos + ivec2(chunk_shift_x, chunk_shift_y),
                index,
            })
        }
    }

    fn movement(&mut self, moves: DataArray<Option<TileInfo>>) {
        for (index, tile_info) in moves
            .into_iter()
            .enumerate()
            .filter_map(|(index, tile_info)| tile_info.map(|tile_info| (index, tile_info)))
        {
            self.tile_info[index] = Some(tile_info);
        }

        for (index, tile) in self.tile_info.iter().enumerate() {
            self.tiles[index] = tile.is_some();
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MoveInfo {
    Impossible,
    Possible,
    Unknown,
}

struct ChunkCalculation {
    checked: DataArray<bool>,
    moves: DataArray<Option<usize>>,
    moves_to: DataArray<Option<TileInfo>>,
    update_tiles: Vec<usize>,
    unknown: DataArray<bool>,
    view_update: DataArray<Option<Option<TileInfo>>>,
}
