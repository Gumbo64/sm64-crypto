mod sm64;
use sm64::{GamePad, SM64Game};

mod sm64_util;


use wasmtime::Error;
fn main() -> Result<(), Error> {
    let mut game = SM64Game::new("../WASM/sm64_headless.us.wasm")?;

    let mut i = 0;
    while i < 2000 {

        let mut button: u16 = 0;
        let stick_x: i8 = 0;
        let mut stick_y: i8 = 0;

        if (150 < i && i < 160) || (200 < i && i < 300) {
            button = sm64_util::START_BUTTON;
        }

        if i > 300 {
            stick_y = 80;
        }

        if i % 2 == 0 {
            button = sm64_util::A_BUTTON;
        }

        let pad = GamePad::new(button, stick_x, stick_y);
        game.step_game(pad)?;


        let state = game.get_game_state()?;
        println!("{}\n", state.to_string());
        i += 1;
    }

    Ok(())
}