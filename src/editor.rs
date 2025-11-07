use anyhow::Result;
use crossterm::event::{Event, KeyCode};
use crossterm::style::Color;
use crossterm::{terminal, cursor::MoveTo, style::{Print, SetForegroundColor, SetBackgroundColor, ResetColor}};
use crossterm::QueueableCommand;
use std::io::Write;

pub enum Actions {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    EnterMode(Mode),
    PrintChar(char),
    Backspace,
    NewLine,
    Save,
    SaveAs(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
}

pub fn handle_normal_event(ev: Event) -> Option<Actions> {
    match ev {
        Event::Key(key) => {
            use crossterm::event::KeyModifiers;
            match (key.code, key.modifiers) {
                (KeyCode::Char('h'), KeyModifiers::NONE) => Some(Actions::MoveLeft),
                (KeyCode::Char('j'), KeyModifiers::NONE) => Some(Actions::MoveDown),
                (KeyCode::Char('k'), KeyModifiers::NONE) => Some(Actions::MoveUp),
                (KeyCode::Char('l'), KeyModifiers::NONE) => Some(Actions::MoveRight),
                (KeyCode::Char('i'), KeyModifiers::NONE) => Some(Actions::EnterMode(Mode::Insert)),
                (KeyCode::Char('s'), KeyModifiers::CONTROL) => Some(Actions::Save),
                (KeyCode::Char('S'), KeyModifiers::CONTROL) => {
                    // For now, just save to a hardcoded path. We'll add proper UI for this later.
                    Some(Actions::SaveAs("new_file.txt".to_string()))
                },
                _ => None,
            }
        },
        _ => None,
    }
}

pub fn handle_insert_event(ev: Event) -> Option<Actions> {
    match ev {
        Event::Key(key) => match key.code {
            KeyCode::Esc => Some(Actions::EnterMode(Mode::Normal)),
            KeyCode::Char(c) => Some(Actions::PrintChar(c)),
            KeyCode::Backspace => Some(Actions::Backspace),
            KeyCode::Enter => Some(Actions::NewLine),
            _ => None,
        },
        _ => None,
    }
}

use crate::buffer::Buffer;

pub struct Editor {
    pub buffer: Buffer,
    pub cx: u16,
    pub cy: u16,
    pub mode: Mode,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Buffer { file: None, lines: vec![String::new()] },
            cx: 0,
            cy: 0,
            mode: Mode::Normal,
        }
    }

    pub fn with_buffer(buffer: Buffer) -> Self {
        Self {
            buffer,
            cx: 0,
            cy: 0,
            mode: Mode::Normal,
        }
    }
    pub fn handle_event(&self, ev: Event) -> Option<Actions> {
        match self.mode {
            Mode::Normal => handle_normal_event(ev),
            Mode::Insert => handle_insert_event(ev),
        }
    }
    pub fn apply_action(&mut self, action: Actions) {
        match action {
            Actions::MoveLeft => {
                if self.cx > 0 { self.cx -= 1; }
            }
            Actions::MoveRight => {
                if let Ok(line) = self.buffer.get_line(self.cy as usize) {
                    let line_len = line.len() as u16;
                    if self.cx < line_len { self.cx += 1; }
                }
            }
            Actions::MoveUp => {
                if self.cy > 0 {
                    self.cy -= 1;
                    if let Ok(line) = self.buffer.get_line(self.cy as usize) {
                        let line_len = line.len() as u16;
                        if self.cx > line_len {
                            self.cx = line_len;
                        }
                    }
                }
            }
            Actions::MoveDown => {
                if (self.cy as usize) + 1 < self.buffer.len() {
                    self.cy += 1;
                    if let Ok(line) = self.buffer.get_line(self.cy as usize) {
                        let line_len = line.len() as u16;
                        if self.cx > line_len {
                            self.cx = line_len;
                        }
                    }
                }
            }
            Actions::EnterMode(m) => self.mode = m,
            Actions::PrintChar(c) => {
                if let Ok(_) = self.buffer.insert_char(self.cy as usize, self.cx as usize, c) {
                    self.cx += 1;
                }
            }
            Actions::Backspace => {
                if self.cx > 0 {
                    if let Ok(_) = self.buffer.remove_char(self.cy as usize, (self.cx - 1) as usize) {
                        self.cx -= 1;
                    }
                } else if self.cy > 0 {
                    if let Ok(prev_line_len) = self.buffer.join_with_previous_line(self.cy as usize) {
                        self.cy -= 1;
                        self.cx = prev_line_len as u16;
                    }
                }
            }
            Actions::NewLine => {
                if let Ok(line) = self.buffer.get_line_mut(self.cy as usize) {
                    let tail = line.split_off(self.cx as usize);
                    self.buffer.lines.insert((self.cy + 1) as usize, tail);
                    self.cy += 1;
                    self.cx = 0;
                }
            }
            Actions::Save => {
                if let Err(e) = self.buffer.save() {
                    // TODO: Show error in status line
                    eprintln!("Error saving file: {}", e);
                }
            }
            Actions::SaveAs(path) => {
                if let Err(e) = self.buffer.save_as(path) {
                    // TODO: Show error in status line
                    eprintln!("Error saving file: {}", e);
                }
            }
        }
    }
    pub fn render(&self, stdout: &mut impl Write) -> Result<()> {
        let (w, h) = terminal::size()?;
        stdout.queue(terminal::Clear(terminal::ClearType::All))?;
        for (i, line) in self.buffer.lines.iter().enumerate() {
            if i as u16 >= h.saturating_sub(1) { break; }
            stdout.queue(MoveTo(0, i as u16))?;
            stdout.queue(Print(line))?;
        }
        let mode_name = match self.mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
        };
        let filename = self.buffer.display_name();
        let line = (self.cy + 1).to_string();
        let col = (self.cx + 1).to_string();
        let percent = if self.buffer.len() <= 1 {
            100
        } else {
            let last = (self.buffer.len() - 1) as f64;
            let pct = (self.cy as f64 / last) * 100.0;
            pct.round() as u16
        };
        let left = format!("{} > {} >", mode_name, filename);
        let right = format!("Ln {} Col {}  {}%", line, col, percent);
        let status_y = h.saturating_sub(1);
        let mut status_line = String::new();
        let left_len = left.len();
        let right_len = right.len();
        let total_width = w as usize;
        if left_len + right_len >= total_width {
            let available = total_width.saturating_sub(left_len + 1);
            status_line.push_str(&left);
            if available > 0 {
                let truncated = &right[..available.min(right.len())];
                status_line.push_str(truncated);
            }
        } else {
            status_line.push_str(&left);
            let pad = total_width.saturating_sub(left_len + right_len);
            for _ in 0..pad { status_line.push(' '); }
            status_line.push_str(&right);
        }
        let bar_bg = Color::DarkGrey;
        let mode_color = match self.mode {
            Mode::Normal => Color::Magenta,
            Mode::Insert => Color::Cyan,
        };
        stdout.queue(MoveTo(0, status_y))?;
        stdout.queue(SetBackgroundColor(bar_bg))?;
        let filler = " ".repeat(total_width);
        stdout.queue(Print(&filler))?;
        stdout.queue(MoveTo(0, status_y))?;
        stdout.queue(SetForegroundColor(mode_color))?;
        stdout.queue(Print(&left))?;
        let right_x = (w as usize).saturating_sub(right.len() as usize) as u16;
        stdout.queue(MoveTo(right_x, status_y))?;
        stdout.queue(Print(&right))?;
        stdout.queue(ResetColor)?;
        let cx = self.cx.min(w.saturating_sub(1));
        let cy = self.cy.min(h.saturating_sub(1));
        stdout.queue(MoveTo(cx, cy))?;
        stdout.flush()?;
        Ok(())
    }
}
               
