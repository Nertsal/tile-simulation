use macroquad::prelude::*;

mod constants;
mod game;
mod update_view;

use game::Game;

const FIXED_DELTA_TIME: f32 = 1.0 / 30.0;
const MAX_UPDATES_PER_FRAME: usize = 5;

#[macroquad::main("Tile Physics")]
async fn main() {
    let mut game = Game::new();

    let mut frame_time = 0.0;
    let mut paused = true;

    let mut fps_timer = 0.0;
    let mut frames = 0;

    loop {
        if is_key_pressed(KeyCode::P) {
            paused = !paused;
        }

        let delta_time = get_frame_time();
        fps_timer += delta_time;
        frames += 1;
        if fps_timer >= 1.0 {
            println!("FPS: {:.1}", frames as f32 / fps_timer);
            fps_timer = 0.0;
            frames = 0;
        }
        game.update(delta_time);

        if !paused || is_key_pressed(KeyCode::Space) {
            frame_time += delta_time;
            let mut updates = 0;
            while !paused && updates < MAX_UPDATES_PER_FRAME && frame_time >= FIXED_DELTA_TIME
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
