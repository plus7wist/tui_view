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
use ratatui::{backend::CrosstermBackend, Terminal};
use view::run_app;

pub use crossterm::event;
pub use view::App;

use std::{io, panic, rc::Rc};
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

/// create_view function expects a Struct that implements this trait.
/// All data and behaviour is transfered into the frontend through this.
pub trait Opts {
    /// This is called on (pretty much) every key event and how the
    /// consumer customizes the app's behaviour. They can match on
    /// the key event and define behaviours accordindly.
    fn keybinds(&self, key: event::KeyEvent, app: App) -> App {
        app
    }
    /// This is supposed to return the actual data to be
    /// loaded into the app as Page structs.
    fn get_pages(&self) -> Vec<Page>;
    /// You can define words here that will take priority in search.
    fn get_keywords(&self) -> Vec<&'static str> {
        vec![]
    }
}

pub fn create_view(opts: Rc<dyn Opts>) -> Result<()> {
    better_panic::install();

    enable_raw_mode()?;
    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    setup_panic_hook();

    let app = App::new(opts);
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

#[derive(Debug, Clone)]
pub struct Page {
    /// The text displayed in the reader.
    pub contents: String,
    /// The text displayed in the dock.
    pub title: String,
    /// A value that will be used to sort filtered
    /// pages in descending order. If None, search
    /// will try to calculate the relevancy of pages
    /// and sort accordingly.
    pub sort_field: Option<f64>,
    relevancy: u64,
}

impl Page {
    pub fn new(contents: String, title: String, sort_field: Option<f64>) -> Self {
        Self {
            contents,
            title,
            sort_field,
            relevancy: 0,
        }
    }

    fn search(&mut self, needle: &str, keywords: Vec<&str>) {
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

                let keyword_multiplier = if keywords.contains(&comb.as_str()) {
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
