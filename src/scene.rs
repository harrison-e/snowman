use crossterm::{
    execute,
    terminal::{size, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{Show, Hide, MoveTo},
};
use std::io::{stdout, Write};
use std::time::Duration;
use chrono::prelude::*;
use clap::ValueEnum;
use rand::{Rng, rngs::ThreadRng};
pub const TIMESTEP: Duration = Duration::from_millis(750); // T
const GRAVITY: f32 = 0.98;  // b/T



/**
 *  A single snowflake
 *  Position is in `b`, or bricks, aka one terminal space
 */
#[derive(Debug)]
struct Snowflake {
    x: f32, // b
    y: f32, // b
    m: f32, // grams
}

impl Snowflake {
    fn new(x0: f32, y0: f32, m: f32) -> Self {
        Snowflake {
            x: x0,
            y: y0,
            m,
        }
    }

    fn update(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy + (GRAVITY * self.m);
    }

    fn is_alive(&self, x_max: f32, y_max: f32) -> bool {
        return self.y <= y_max && self.x <= x_max && self.x >= 0f32;
    }
}



/**
 *  Intensity enum is used to construct a Scene 
 *  
 */
#[derive(Debug, Clone, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum SnowfallIntensity {
    #[value(name = "low", alias = "l")]
    Low,
    #[value(name = "medium", alias = "m")]
    Medium,
    #[value(name = "high", alias = "h")]
    High
}

/**
 *  Struct containing all of the scene's data
 */
pub struct Scene {
    cols: u16,                  // b
    rows: u16,                  // b
    snowman_col: Option<u16>,   // b
    tree_col: Option<u16>,      // b
    santa_col: Option<u16>,     // b
    rng: ThreadRng,
    snowflakes: Vec<Snowflake>,
    max_snowflakes: usize,
    intensity: SnowfallIntensity,
    wind_x: f32,                // b/T
    wind_y: f32,                // b/T
} 

impl Scene {
    fn calc_entity_positions(&mut self) {
        // Ensure that tree, snowman, and Santa don't overlap
        if self.cols >= 6 {
            self.snowman_col = Some(self.rng.gen_range(1..self.cols-1));
        }
        
        if let Some(snowman_col) = self.snowman_col {
            if self.cols >= 12 {
                let tree_col = loop {
                    let num: u16 = self.rng.gen_range(2..self.cols-2);
                    if (num as i32 - snowman_col as i32).abs() >= 6 {
                        break num;
                    }
                };
                self.tree_col = Some(tree_col);

                if self.cols >= 17 {
                    let santa_col = loop {
                        let num: u16 = self.rng.gen_range(1..self.cols-1);
                        if (num as i32 - snowman_col as i32).abs() >= 5
                            && (num as i32 - tree_col as i32).abs() >= 6 {
                            break num;
                        }
                    };
                    self.santa_col = Some(santa_col);
                }
            }
        }
    }

    fn init_snowflakes(&mut self) {
        self.max_snowflakes = (match self.intensity {
            SnowfallIntensity::Low => 0.05,
            SnowfallIntensity::Medium => 0.1,
            SnowfallIntensity::High => 0.2,
        } * (self.cols as f32) * (self.rows as f32)) as usize;
        let num_snowflakes = self.rng.gen_range(0..(self.max_snowflakes / 16));
        self.snowflakes = (0..num_snowflakes)
            .map(|_i| 
                Snowflake::new(
                    self.rng.gen_range(0..self.cols).into(), 
                    0.0,
                    self.rng.gen_range(0.6..=1.4),
                ))
            .collect();
    }

    pub fn new(intensity: SnowfallIntensity) -> Self {
        let (c, r) = size().expect("Could not get terminal size.");
        let mut rng = rand::thread_rng();

        let mut s = Scene {
            cols: c,
            rows: r,
            snowman_col: None, 
            tree_col: None, 
            santa_col: None, 
            rng: rng.clone(),
            snowflakes: Vec::new(),
            max_snowflakes: 0usize,
            intensity,
            wind_x: rng.gen_range(-0.25..0.25),
            wind_y: rng.gen_range(0.0..0.05),
        };
        s.calc_entity_positions();
        s.init_snowflakes();
        s
    }
    
    pub fn update(&mut self) {
        // Update scene if dimensions are changed
        let (c, r) = size().expect("Could not get terminal size.");
        if c != self.cols || r != self.rows {
            self.cols = c;
            self.rows = r;
            self.calc_entity_positions();
            self.init_snowflakes();
            return;
        }

        // Update wind with 1/5 chance 
        if self.rng.gen_ratio(1, 5) {
            self.wind_x += self.rng.gen_range(-0.1..=0.1);
            self.wind_y += self.rng.gen_range(-0.1..=0.1);
            self.wind_y = self.wind_y.max(0.0); // So snow doesn't go up
        }

        // Update snowflakes
        for snowflake in &mut self.snowflakes {
            snowflake.update(self.wind_x, self.wind_y);
        }

        // Remove snowflakes out of bounds 
        self.snowflakes.retain(|s| s.is_alive(self.cols.into(), self.rows.into()));

        // Add new snowflakes randomly
        // TODO better calibrate new num snowflakes with cols, rows
        if self.snowflakes.len() < self.max_snowflakes {
            let rem: usize = self.max_snowflakes - self.snowflakes.len();
            let num_new: usize = self.rng.gen_range(0..=(rem / 10));
            self.snowflakes.extend(
                (0..num_new)
                    .map(|_i| 
                        Snowflake::new(
                             self.rng.gen_range(0..self.cols).into(), 
                             0.0,
                             self.rng.gen_range(0.6..1.4),
                        ))
                    .collect::<Vec<Snowflake>>()
            );
        } 
    }

    pub fn render(&mut self) {
        self.clear_screen();

        self.render_snow();

        self.render_snowman();

        self.render_tree();

        if self.days_until_xmas() == 0i64 {
            self.render_santa();
        }

        self.render_time();

        // Render snowy ground
        self.move_cursor(0, self.rows-1);
        print!("\x1B[47m{0}\x1B[0m", (0..self.cols).map(|_| ' ').collect::<String>());

        // Flush
        stdout().flush().expect("Could not flush stdout.");
    }

    fn render_snow(&self) {
        for snowflake in &self.snowflakes {
            self.move_cursor(snowflake.x as u16, snowflake.y as u16);
            print!("\x1B[37m{0}\x1B[0m", if snowflake.m <= 0.9 { '+' } else { '*' });
        }
    }

    // This is the snowman:
    // _XX_
    //  ''- 
    // -X:-
    //  X:
    //  This is some janky cursor-moving, ANSI-color-encoding, inline-printing code 
    fn render_snowman(&self) {
        if let Some(snowman_col) = self.snowman_col {
            self.move_cursor(snowman_col-1, self.rows-5);
            print!("\x1B[0;30m_\x1B[0;40m  \x1B[0;30m_\x1B[0m");
            self.move_cursor(snowman_col, self.rows-4);
            print!("\x1B[47;30m''\x1B[0;38;5;202m>\x1B[0m");
            self.move_cursor(snowman_col-1, self.rows-3);
            print!("\x1B[0;38;5;52m\\\x1B[0;47;30m :\x1B[0;38;5;52m/\x1B[0m");
            self.move_cursor(snowman_col, self.rows-2);
            print!("\x1B[0;47;30m :\x1B[0m");
        }
    }

    // This is the tree:
    //   *
    //  _X_
    // _XXX_
    // XXXXX
    //   X
    // This, like the above function, is super jank
    fn render_tree(&mut self) {
        if let Some(tree_col) = self.tree_col {
            self.move_cursor(tree_col, self.rows-6);
            print!("\x1B[0;33m*\x1B[0m");
            self.move_cursor(tree_col-1, self.rows-5);
            print!("\x1B[0;37m_\x1B[0;42m \x1B[0;37m_\x1B[0m");
            self.move_cursor(tree_col-2, self.rows-4);
            print!("\x1B[0;37m_\x1B[0;42m   \x1B[0;37m_\x1B[0m");
            self.move_cursor(tree_col-2, self.rows-3);
            print!("\x1B[0;42m     \x1B[0m");
            if self.days_until_xmas() == 0i64 {
                self.move_cursor(tree_col-1, self.rows-2);
                print!("\x1B[0;33;44m┼\x1B[0;48;5;52m \x1B[0;33;41m┼\x1B[0m");
            }
            else {
                self.move_cursor(tree_col, self.rows-2);
                print!("\x1B[0;48;5;52m \x1B[0m");
            }
        }
    }

    // This is Santa:
    //  __*
    //  XX
    // sXXz
    //  XX
    fn render_santa(&self) {
        if let Some(santa_col) = self.santa_col {
            self.move_cursor(santa_col, self.rows-5);
            print!("\x1B[0;41;97m/\\\x1B[0;97m*\x1B[0m");
            self.move_cursor(santa_col, self.rows-4);
            print!("\x1B[0;107;30m^^\x1B[0m");
            self.move_cursor(santa_col-1, self.rows-3);
            print!("\x1B[0;31m/\x1B[0;41;97m  \x1B[0;31m\\\x1B[0m");
            self.move_cursor(santa_col, self.rows-2);
            print!("\x1B[0;41;97m  \x1B[0m");
        }
    }

    fn render_time(&mut self) {
        // Do xmas math
        let now = Local::now();
        let days_until_xmas = self.days_until_xmas();

        // Create format strings
        let now_str = now.format("%b %d, %Y - %I:%M%p").to_string();
        let xmas_str = if days_until_xmas > 0i64 {
            format!("{0} days until Christmas.", days_until_xmas)
        } else {
            String::from("Merry Christmas!")
        };

        // Render xmas clock 
        let border_len = now_str.len().max(xmas_str.len()) + 2; // +2 for spaces
        self.move_cursor(0, 0);
        print!("╭{0}╮", (0..border_len).map(|_i| '─').collect::<String>());
        self.move_cursor(0, 1);
        print!("│ {0}{1} │", now_str, (0..(border_len - 2 - now_str.len())).map(|_i| ' ').collect::<String>());
        self.move_cursor(0, 2);
        print!("│ {0}{1} │", xmas_str, (0..(border_len - 2 - xmas_str.len())).map(|_i| ' ').collect::<String>());
        self.move_cursor(0, 3);
        print!("╰{0}╯", (0..border_len).map(|_i| '─').collect::<String>());
    }

    fn move_cursor(&self, col: u16, row: u16) {
        execute!(
            stdout(),
            MoveTo(col, row)
        ).unwrap();
    }

    fn clear_screen(&self) {
        execute!(
            stdout(),
            Clear(ClearType::All),
            MoveTo(0, 0)
        ).expect("Could not clear screen.");
    }

    fn days_until_xmas(&self) -> i64 {
        let now = Local::now().date_naive();
        let xmas = NaiveDate::from_ymd_opt(now.year(), 12, 25).unwrap();
        let xmas = if xmas < now {
            NaiveDate::from_ymd_opt(now.year() + 1, 12, 25).unwrap()
        } else {
            xmas
        };
        xmas.signed_duration_since(now).num_days()
    }
    
    pub fn enter(&mut self) {
        execute!(
            stdout(),
            EnterAlternateScreen,
            Hide
        ).expect("Could not enter alternate screen.")
    }
    
    pub fn exit(&mut self) {
        execute!(
            stdout(),
            LeaveAlternateScreen,
            Show
        ).expect("Could not enter alternate screen.")
    }
}
