mod page;
mod view;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use itertools::Itertools;
use tui::{backend::CrosstermBackend, Terminal};
use view::{run_app, App};

use std::{io, panic};
fn cleanup_terminal() {
    let mut stdout = io::stdout();

    execute!(stdout, cursor::MoveTo(0, 0)).unwrap();
    execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();

    execute!(stdout, terminal::LeaveAlternateScreen).unwrap();
    execute!(stdout, cursor::Show).unwrap();

    terminal::disable_raw_mode().unwrap();
}

fn setup_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        cleanup_terminal();
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Opts {
    fn on_enter(&self, selected_item: &Page) -> Result<()>;
    fn get_pages(&self) -> Vec<Page>;
    fn get_keywords(&self) -> Option<Vec<&'static str>>;
}

pub fn create_view(opts: Box<dyn Opts>) -> Result<()> {
    better_panic::install();

    enable_raw_mode()?;
    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    setup_panic_hook();

    let app = App::new(opts)?;
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

#[derive(Clone)]
pub struct Page {
    pub contents: String,
    pub title: String,
    pub relevancy: u64,
    pub sort_field: Option<f64>,
}

impl Page {
    fn search(&mut self, needle: &str, keywords: Option<Vec<&str>>) {
        let haystack = &self.contents;
        let mut relevancy = 0;

        let words = needle.split(' ').collect::<Vec<&str>>();

        let mut combinations = Vec::new();

        let min_combs = words.len().checked_sub(3).unwrap_or(1);
        let max_combs = words.len() + 1;
        for k in min_combs..max_combs {
            let combs = words.iter().combinations(k);
            for c in combs {
                let combination = c
                    .iter()
                    .map(|w| w.to_string())
                    .join(" ")
                    .to_lowercase()
                    .trim()
                    .to_string();

                // This check might be unnecessary
                if !combinations.contains(&combination) {
                    combinations.push(combination);
                }
            }
        }

        for comb in combinations {
            if haystack.contains(&comb) {
                let needle_size_multiplier = (comb.split(' ').count() as u64).pow(5);

                let keyword_multiplier =
                    if keywords.is_some() && keywords.clone().unwrap().contains(&comb.as_str()) {
                        10
                    } else {
                        1
                    };

                let count_relevancy = haystack.matches(&comb).count() as u64;

                let title_relevancy = 25 * self.title.matches(&comb).count() as u64;

                relevancy += keyword_multiplier
                    * needle_size_multiplier
                    * (count_relevancy + title_relevancy);
            }
        }

        self.relevancy = relevancy;
    }
}