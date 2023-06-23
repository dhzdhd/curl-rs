use regex::Regex;
use serde_json::Value;
use tui::style::{Color, Style};
use tui_textarea::TextArea;

use crate::traits::Tab;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppMode {
    Request = 0,
    Response = 1,
}

impl Tab for AppMode {
    fn as_int(&self) -> u8 {
        *self as u8
    }

    fn to_enum(&self, num: u8) -> Self {
        match num {
            0 => Self::Request,
            1 => Self::Response,
            _ => Self::Request,
        }
    }

    fn next(&self) -> Self {
        self.to_enum((self.as_int() + 1) % 2)
    }

    fn previous(&self) -> Self {
        self.next()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    UriEditing = 0,
    Normal = 1,
    PayloadEditing = 2,
}

impl Tab for InputMode {
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

pub struct Response {
    pub json: String,
    pub status: u32,
}

pub struct State<'a> {
    pub payload_titles: Vec<&'a str>,
    pub req_tab_index: usize,
    pub main_index: usize,
    pub input_mode: InputMode,
}

impl<'a> State<'a> {
    pub fn new() -> Self {
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

pub struct Editor<'a> {
    pub title: &'a str,
    pub text_area: TextArea<'a>,
}

impl<'a> Editor<'a> {
    pub fn default(title: &'a str) -> Self {
        let mut text_area = TextArea::default();
        text_area.set_style(Style::default().bg(Color::Black).fg(Color::White));

        Self { title, text_area }
    }

    pub fn text(&self) -> String {
        self.text_area.lines().join("\n")
    }

    pub fn validate_uri(&self) -> bool {
        let url_pattern = r#"^(https?|ftp):\/\/[^\s/$.?#].[^\s]*$"#;
        let re = Regex::new(url_pattern).unwrap();

        !self.text().trim().is_empty() && re.is_match(self.text().as_str())
    }

    pub fn validate_json(&self) -> bool {
        let parsed_json: Result<Value, serde_json::Error> =
            serde_json::from_str(self.text().as_str());
        parsed_json.is_ok()
    }
}

pub struct Request {
    pub headers: Option<String>,
    pub body: Option<String>,
    pub uri: String,
    pub method: String,
}

impl Request {
    pub async fn fetch() {}
}
