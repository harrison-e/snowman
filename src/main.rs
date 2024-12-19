mod scene;
use scene::*;
use clap::Parser; 
use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode},
    event::{self, Event, KeyCode, poll},
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'i', long = "intensity", value_enum, default_value_t = SnowfallIntensity::Medium)]
    intensity: SnowfallIntensity,
}

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
    let args = Args::parse();
    let mut scene = Scene::new(args.intensity);
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
