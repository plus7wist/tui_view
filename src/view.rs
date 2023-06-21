use std::{io, rc::Rc, time::Duration};
use tui::{layout::Rect, widgets::Clear};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Row, Table, TableState, Wrap},
    Frame, Terminal,
};

use crate::Opts;
use crate::Page;

#[derive(Clone)]
pub struct App {
    pub state: TableState,
    pub pages: Vec<Page>,
    pub current_pages: Vec<Page>,
    scroll: u16,
    pub search: Vec<char>,
    latest_search: Vec<char>,
    show_dock: bool,
    pub show_popup: bool,
    pub popup_content: String,
    opts: Rc<dyn Opts>,
}

impl App {
    pub fn new(opts: Rc<dyn Opts>) -> Self {
        App {
            state: TableState::default(),
            current_pages: opts.get_pages(),
            scroll: 0,
            search: Vec::new(),
            pages: opts.get_pages(),
            latest_search: Vec::new(),
            show_dock: true,
            show_popup: false,
            popup_content: String::default(),
            opts,
        }
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

    fn toggle_dock(&mut self) {
        self.show_dock = !self.show_dock;
    }

    fn toggle_popup(&mut self) {
        self.show_popup = !self.show_popup;
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
                        KeyCode::Char('b') => app.toggle_dock(),
                        KeyCode::Char('p') => app.toggle_popup(),
                        _ => {}
                    },
                    _ => {
                        if key.code == KeyCode::Backspace {
                            app.search.pop();
                        }
                        if let KeyCode::Char(x) = key.code {
                            app.search.push(x);
                        }
                    }
                }

                if key.modifiers == event::KeyModifiers::CONTROL
                    && key.code == event::KeyCode::Char('s')
                {
                    app = app.opts.keybinds(key, app.clone());
                }
            }
        } else if app.search != app.latest_search {
            app.latest_search = app.search.clone();
            app.search(app.search.iter().collect());
            app.state.select(Some(0));
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let dock_size = if app.show_dock { 30 } else { 0 };
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(dock_size),
            Constraint::Percentage(70),
        ])
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

    f.render_widget(page, main_layout[1]);
    f.render_widget(search, layout[0]);
    f.render_stateful_widget(directory_table, layout[1], &mut app.state);
    if app.show_popup {
        let size = f.size();
        let popup = Paragraph::new(app.popup_content.clone())
            .block(Block::default().title("Popup").borders(Borders::ALL))
            .wrap(Wrap { trim: false });
        let area = centered_rect(60, 20, size);
        f.render_widget(Clear, area);
        f.render_widget(popup, area);
    }
}
