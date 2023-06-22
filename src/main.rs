use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use regex::Regex;
use serde_json::Value;

use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{self, Span, Spans, Text},
    widgets::{Block, Borders, Tabs},
    Frame, Terminal,
};
use tui_textarea::TextArea;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InputMode {
    UriEditing = 0,
    Normal = 1,
    PayloadEditing = 2,
}

impl InputMode {
    fn as_int(&self) -> u8 {
        *self as u8
    }

    fn to_enum(&self, num: u8) -> Self {
        match num {
            0 => Self::UriEditing,
            1 => Self::Normal,
            2 => Self::PayloadEditing,
            _ => Self::Normal,
        }
    }

    fn next(&self) -> Self {
        self.to_enum((self.as_int() + 1) % 3)
    }

    fn previous(&self) -> Self {
        self.to_enum((self.as_int() + 2) % 3)
    }
}

struct Response {
    json: String,
    status: u32,
}

struct State<'a> {
    pub payload_titles: Vec<&'a str>,
    pub req_tab_index: usize,
    pub main_index: usize,
    pub input_mode: InputMode,
}

impl<'a> State<'a> {
    fn new() -> Self {
        Self {
            payload_titles: vec!["Headers", "Body"],
            req_tab_index: 0,
            main_index: 0,
            input_mode: InputMode::UriEditing,
        }
    }

    pub fn next_payload(&mut self) {
        self.req_tab_index = (self.req_tab_index + 1) % self.payload_titles.len();
    }

    pub fn previous_payload(&mut self) {
        if self.req_tab_index > 0 {
            self.req_tab_index -= 1;
        } else {
            self.req_tab_index = self.payload_titles.len() - 1;
        }
    }
}

struct Editor<'a> {
    title: &'a str,
    text_area: TextArea<'a>,
}

impl<'a> Editor<'a> {
    fn default(title: &'a str) -> Self {
        let mut text_area = TextArea::default();
        text_area.set_style(Style::default().bg(Color::Black).fg(Color::White));

        Self { title, text_area }
    }

    fn text(&self) -> String {
        self.text_area.lines().join("\n")
    }

    fn validate_uri(&self) -> bool {
        let url_pattern = r#"^(https?|ftp):\/\/[^\s/$.?#].[^\s]*$"#;
        let re = Regex::new(url_pattern).unwrap();

        !self.text().trim().is_empty() && re.is_match(self.text().as_str())
    }

    fn validate_json(&self) -> bool {
        let parsed_json: Result<Value, serde_json::Error> =
            serde_json::from_str(self.text().as_str());
        parsed_json.is_ok()
    }
}

struct App<'a> {
    uri_editor: Editor<'a>,
    payload_editors: Vec<Editor<'a>>,
    state: State<'a>,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl<'a> App<'a> {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            uri_editor: Editor::default("uri"),
            payload_editors: vec![Editor::default("headers"), Editor::default("body")],
            state: State::new(),
            terminal,
        })
    }

    fn run(&mut self) -> io::Result<()> {
        loop {
            // Try to make ui() a struct method and not an assoc method
            self.terminal.draw(|f| {
                Self::ui(
                    f,
                    &self.state,
                    &mut self.uri_editor,
                    &mut self.payload_editors,
                )
            })?;

            let event = event::read()?;
            if let Event::Key(key) = event.into() {
                if key.kind == KeyEventKind::Press {
                    match self.state.input_mode {
                        InputMode::PayloadEditing => {
                            self.payload_editors[self.state.req_tab_index]
                                .text_area
                                .input(key);
                        }
                        InputMode::UriEditing => {
                            self.uri_editor.text_area.input(key);
                        }
                        _ => {}
                    }

                    match key.modifiers {
                        KeyModifiers::NONE => match self.state.input_mode {
                            InputMode::Normal => match key.code {
                                KeyCode::Right => self.state.next_payload(),
                                KeyCode::Left => self.state.previous_payload(),
                                _ => {}
                            },
                            _ => {}
                        },
                        KeyModifiers::ALT => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            _ => {}
                        },
                        KeyModifiers::SHIFT => match key.code {
                            KeyCode::Down => self.state.input_mode = self.state.input_mode.next(),
                            KeyCode::Up => self.state.input_mode = self.state.input_mode.previous(),
                            KeyCode::Enter => {}
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }
    }

    fn ui(
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        state: &State,
        uri_editor: &mut Editor<'a>,
        payload_editors: &mut Vec<Editor<'a>>,
    ) {
        let size = f.size();

        // Layouts
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(size);

        let req_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(main_layout[0]);

        let resp_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(main_layout[1]);

        let block = Block::default().style(Style::default().bg(Color::Black).fg(Color::White));
        f.render_widget(block, size);

        uri_editor.text_area.set_block(
            Block::default()
                .borders(Borders::all())
                .border_style(
                    Style::default().fg(if state.input_mode == InputMode::UriEditing {
                        Color::Cyan
                    } else {
                        Color::White
                    }),
                )
                .title(uri_editor.title),
        );

        // Payload tabs
        let payload_titles = state
            .payload_titles
            .iter()
            .map(|t| {
                let (first, rest) = t.split_at(1);
                Spans::from(vec![
                    Span::styled(first, Style::default().fg(Color::Yellow)),
                    Span::styled(rest, Style::default().fg(Color::Green)),
                ])
            })
            .collect();

        let tabs = Tabs::new(payload_titles)
            .block(Block::default().borders(Borders::ALL).title("option"))
            .select(state.req_tab_index)
            .style(
                Style::default().fg(if state.input_mode == InputMode::Normal {
                    Color::Cyan
                } else {
                    Color::White
                }),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Blue),
            );

        // Payload editor
        let inner = &mut payload_editors[state.req_tab_index];
        inner.text_area.set_block(
            Block::default()
                .borders(Borders::all())
                .border_style(Style::default().fg(
                    if state.input_mode == InputMode::PayloadEditing {
                        if inner.validate_json() {
                            Color::Cyan
                        } else {
                            Color::Red
                        }
                    } else {
                        Color::White
                    },
                ))
                .title(inner.title),
        );

        f.render_widget(uri_editor.text_area.widget(), req_layout[0]);
        f.render_widget(tabs, req_layout[1]);
        f.render_widget(inner.text_area.widget(), req_layout[2]);
    }
}

impl<'a> Drop for App<'a> {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
        self.terminal.show_cursor().unwrap();
    }
}

fn main() -> io::Result<()> {
    App::new()?.run()
}
