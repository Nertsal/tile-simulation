use macroquad::prelude::IVec2;
use std::collections::HashMap;

use crate::game::tile::TileInfo;

#[derive(Default)]
pub struct UpdateView {
    tiles: HashMap<IVec2, Option<TileInfo>>,
}

impl UpdateView {
    pub fn into_tiles(self) -> impl Iterator<Item = (IVec2, Option<TileInfo>)> {
        self.tiles.into_iter()
    }

    pub fn update_view(&mut self, tiles: impl Iterator<Item = (IVec2, Option<TileInfo>)>) {
        self.tiles.extend(tiles);
    }

    pub fn update_tile(&mut self, tile_pos: IVec2, tile: Option<TileInfo>) {
        self.tiles.insert(tile_pos, tile);
    }
}
