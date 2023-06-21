use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

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

struct App<'a> {
    pub titles: Vec<&'a str>,
    pub payload_inputs: Vec<String>,
    pub uri_input: String,
    pub index: usize,
    pub input_mode: InputMode,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            titles: vec!["Headers", "Body"],
            payload_inputs: vec!["".to_string(), "".to_string()],
            uri_input: "".to_string(),
            index: 0,
            input_mode: InputMode::UriEditing,
        }
    }

    pub fn next_payload(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous_payload(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // println!("{:?}", app.input_mode.next());

                match key.modifiers {
                    KeyModifiers::NONE => match app.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Right => app.next_payload(),
                            KeyCode::Left => app.previous_payload(),
                            _ => {}
                        },
                        InputMode::PayloadEditing => match key.code {
                            KeyCode::Char(c) => app.payload_inputs[app.index].push(c),
                            KeyCode::Backspace => {
                                app.payload_inputs[app.index].pop();
                            }
                            KeyCode::Enter => app.payload_inputs[app.index].push('\n'),
                            _ => {}
                        },
                        InputMode::UriEditing => match key.code {
                            KeyCode::Char(c) => app.uri_input.push(c),
                            KeyCode::Backspace => {
                                app.uri_input.pop();
                            }
                            _ => {}
                        },
                    },
                    KeyModifiers::ALT => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    },
                    KeyModifiers::SHIFT => match key.code {
                        KeyCode::Down => app.input_mode = app.input_mode.next(),
                        KeyCode::Up => app.input_mode = app.input_mode.previous(),
                        KeyCode::Char(c) => match app.input_mode {
                            InputMode::UriEditing => app.uri_input.push(c),
                            InputMode::PayloadEditing => app.payload_inputs[app.index].push(c),
                            _ => {}
                        },
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(size);

    let block = Block::default().style(Style::default().bg(Color::Black).fg(Color::White));
    f.render_widget(block, size);

    let uri = Paragraph::new(app.uri_input.clone())
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("uri"))
        .alignment(Alignment::Left);
    f.render_widget(uri, layout[0]);

    let titles = app
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(first, Style::default().fg(Color::Yellow)),
                Span::styled(rest, Style::default().fg(Color::Green)),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("option"))
        .select(app.index)
        // .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Blue),
        );
    f.render_widget(tabs, layout[1]);

    let inner = match app.index {
        0 => Paragraph::new(app.payload_inputs[0].clone())
            .style(Style::default().bg(Color::Black).fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("payload"))
            .alignment(Alignment::Left),
        1 => Paragraph::new(app.payload_inputs[1].clone())
            .style(Style::default().bg(Color::Black).fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("payload"))
            .alignment(Alignment::Left),
        _ => unreachable!(),
    };
    f.render_widget(inner, layout[2]);

    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::UriEditing => f.set_cursor(
            layout[0].x + 1 + app.uri_input.width() as u16,
            layout[0].y + 1,
        ),
        InputMode::PayloadEditing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                layout[2].x
                    + app.payload_inputs[app.index]
                        .lines()
                        .last()
                        .unwrap_or("")
                        .width() as u16
                    + 1,
                layout[2].y
                    + app.payload_inputs[app.index]
                        .lines()
                        .collect::<Vec<&str>>()
                        .len() as u16,
            )
        }
    }
}
