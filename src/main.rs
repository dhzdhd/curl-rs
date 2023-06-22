use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Frame, Terminal,
};
use tui_textarea::TextArea;

#[derive(Clone, Copy, Debug)]
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

struct App<'a> {
    uri_editor: TextArea<'a>,
    payload_editors: Vec<TextArea<'a>>,
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
            uri_editor: TextArea::new(vec!["".to_string()]),
            payload_editors: vec![
                TextArea::new(vec!["".to_string()]),
                TextArea::new(vec!["".to_string()]),
            ],
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
                            self.payload_editors[self.state.req_tab_index].input(key);
                        }
                        InputMode::UriEditing => {
                            self.uri_editor.input(key);
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
        uri_editor: &mut TextArea<'a>,
        payload_editors: &mut Vec<TextArea<'a>>,
    ) {
        let size = f.size();

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

        uri_editor.set_block(Block::default().borders(Borders::ALL).title("uri"));
        uri_editor.set_style(Style::default().bg(Color::Black).fg(Color::White));

        f.render_widget(uri_editor.widget(), req_layout[0]);

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
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Blue),
            );

        f.render_widget(tabs, req_layout[1]);

        let inner = match state.req_tab_index {
            0 => &payload_editors[0],
            1 => &payload_editors[1],
            _ => unreachable!(),
        };
        f.render_widget(inner.widget(), req_layout[2]);
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
