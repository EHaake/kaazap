use std::{fmt, time::Duration};

use crate::{config::Config, frame::Frame};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MenuItem {
    StartGame,
    HowToPlay,
}

#[derive(Debug)]
pub struct MenuState {
    selected: MenuItem,
    time_accumulated: Duration,
    title_text: Vec<&'static str>
}

impl MenuState {
    pub fn new() -> Self {
        let title_art = include_str!("../assets/kaazap_title.txt");
        let title_text = title_art.lines().collect::<Vec<&'static str>>();

        Self {
            selected: MenuItem::StartGame,
            time_accumulated: Duration::from_millis(0),
            title_text,
        }
    }

    /// Draw Text Helper
    /// 
    /// Takes the text to draw, location coords and frame to draw into
    fn draw_text(&self, text: &str, x: usize, y: usize, frame: &mut Frame) {
        for (i, ch) in text.chars().enumerate() {
            frame[x + i][y] = ch;
        }
    }

    /// Draw the title which is a Vector<&'static str>
    ///
    /// Iterate through each line and send it to draw_text
    fn draw_title(&self, x: usize, y: usize, frame: &mut Frame) {
        for (row, line) in self.title_text.iter().enumerate() {
            self.draw_text(line, x, y + row, frame); 
        }       
    }

    fn draw_menu_items(&self, x: usize, y: usize, frame: &mut Frame) {

    }

    /// Main draw fn figures out where to render each element, then sends it out
    ///
    pub fn draw(&self, frame: &mut Frame, config: &Config) {
        // TODO: stop using magic numbers for positioning
        let mid = config.num_cols / 2;
        let mut padding_x = self.title_text[1].len() / 2 - 11;
        let padding_y = 2;

        self.draw_title(mid - padding_x, padding_y, frame);

        padding_x = self.
        self.draw_menu_items(x, y, frame);
    }

    /// Accumulate time up to duration to drive menu animations
    ///
    pub fn tick(&mut self, dt: Duration) {
        self.time_accumulated += dt;
        if self.time_accumulated >= Duration::from_millis(350) {
            // toggle anim status
            // anim_state.toggle();
            self.time_accumulated -= Duration::from_millis(350);
        }
    }
}

impl Default for MenuState {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MenuItem::StartGame => write!(f, "Start Game"),
            MenuItem::HowToPlay => write!(f, "How To Play"),
        }
    }
}
