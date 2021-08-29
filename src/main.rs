use macroquad::prelude::*;

mod constants;
mod game;
mod update_view;

use game::Game;

const FIXED_DELTA_TIME: f32 = 1.0 / 30.0;
const MAX_UPDATES_PER_FRAME: usize = 1;

#[macroquad::main("Tile Physics")]
async fn main() {
    let mut game = Game::new();
    let mut frame_time = 0.0;
    let mut paused = true;
    loop {
        if is_key_pressed(KeyCode::P) {
            paused = !paused;
        }

        if !paused || is_key_pressed(KeyCode::Space) {
            let delta_time = get_frame_time();
            frame_time += delta_time;
            game.update(delta_time);
            let mut updates = 0;
            while frame_time >= FIXED_DELTA_TIME && updates < MAX_UPDATES_PER_FRAME
                || paused && updates == 0
            {
                game.fixed_update(FIXED_DELTA_TIME);
                frame_time -= FIXED_DELTA_TIME;
                updates += 1;
            }
        }

        game.draw();
        next_frame().await;
    }
}
