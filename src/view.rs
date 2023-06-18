use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Row, Table, TableState, Wrap},
    Frame, Terminal,
};

use crate::Page;
use crate::{Opts, Result};

pub struct App {
    state: TableState,
    pages: Vec<Page>,
    current_pages: Vec<Page>,
    scroll: u16,
    search: Vec<char>,
    latest_search: Vec<char>,
    opts: Box<dyn Opts>,
}

impl App {
    pub fn new(opts: Box<dyn Opts>) -> Result<Self> {
        Ok(App {
            state: TableState::default(),
            current_pages: opts.get_pages(),
            scroll: 0,
            search: Vec::new(),
            pages: opts.get_pages(),
            latest_search: Vec::new(),
            opts,
        })
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i < self.current_pages.len() - 1 {
                    i + 1
                } else {
                    i
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll = 0;
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll = 0;
    }

    fn load(&mut self) -> String {
        match self.state.selected() {
            Some(i) => match self.current_pages.get(i) {
                Some(x) => x.contents.clone(),
                None => String::from(""),
            },
            None => String::from(""),
        }
    }

    fn scroll_down(&mut self) {
        self.scroll = self.scroll.checked_add(1).unwrap_or(self.scroll);
    }

    fn scroll_up(&mut self) {
        self.scroll = self.scroll.checked_sub(1).unwrap_or(self.scroll);
    }

    fn search(&mut self, search_phrase: String) {
        let search_phrase = search_phrase.trim().to_lowercase();

        if search_phrase.is_empty() {
            self.current_pages = self.pages.clone();
            return;
        }

        self.current_pages = self
            .pages
            .iter_mut()
            .map(|page| {
                page.search(&search_phrase, self.opts.get_keywords());
                page.clone()
            })
            .filter(|md_file| md_file.relevancy > 0)
            .collect();

        self.current_pages.sort_by(|a, b| {
            if let Some(_sort_field) = a.sort_field {
                b.sort_field.partial_cmp(&a.sort_field).unwrap()
            } else {
                b.relevancy.cmp(&a.relevancy)
            }
        });
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.modifiers {
                    KeyModifiers::CONTROL => match key.code {
                        KeyCode::Char('e') => return Ok(()),
                        KeyCode::Char('d') => app.scroll_down(),
                        KeyCode::Char('u') => app.scroll_up(),
                        KeyCode::Char('j') => app.next(),
                        KeyCode::Char('k') => app.previous(),
                        _ => {}
                    },
                    _ => {
                        if key.code == KeyCode::Backspace {
                            app.search.pop();
                        }

                        if let KeyCode::Char(x) = key.code {
                            app.search.push(x);
                        }

                        if key.code == KeyCode::Enter {
                            let index = app.state.selected().unwrap();
                            let selected = app.current_pages.get(index).unwrap();
                            let _ = app.opts.on_enter(selected);
                        }
                    }
                }
            }
        } else if app.search != app.latest_search {
            app.latest_search = app.search.clone();
            app.search(app.search.iter().collect());
            app.state.select(Some(0));
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(f.size());

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Percentage(90)])
        .split(main_layout[0]);

    let mut directory_rows = vec![];

    app.current_pages.iter().for_each(|chapter| {
        let title = chapter.title.clone();
        directory_rows.push(Row::new(vec![title]))
    });

    let directory_table = Table::new(directory_rows)
        .block(Block::default().title("Directory").borders(Borders::ALL))
        .widths(&[Constraint::Percentage(100)])
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let page = Paragraph::new(app.load())
        .block(Block::default().title("Reader").borders(Borders::ALL))
        .scroll((app.scroll, 0))
        .wrap(Wrap { trim: false });

    let input_string: String = app.search.iter().collect();
    let lines = Text::from(input_string.as_str());
    let search = Paragraph::new(lines)
        .block(Block::default().title("Search").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(search, layout[0]);
    f.render_widget(page, main_layout[1]);
    f.render_stateful_widget(directory_table, layout[1], &mut app.state);
}
