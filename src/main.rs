mod scene;
use scene::*;
use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode},
    event::{self, Event, KeyCode, poll},
};

const NUM_SNOWFLAKES: usize = 128;

fn check_quit() -> Result<bool, Box<dyn std::error::Error>> {
    if poll(TIMESTEP)? {
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn main() {
    let mut scene = Scene::new(NUM_SNOWFLAKES);
    enable_raw_mode().expect("Could not enable raw mode.");

    scene.enter();
    loop {
        scene.update();
        scene.render();

        // Check for quit signal and sleep
        if let Ok(true) = check_quit() {
            break;
        } 
    }
    scene.exit();

    disable_raw_mode().expect("Could not disable raw mode.");
}
