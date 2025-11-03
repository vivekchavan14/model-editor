use std::io::{stdout, Write};

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::event::{Event, KeyCode};
use crossterm::style::Print;
use crossterm::{event::read, ExecutableCommand, QueueableCommand, terminal};

/// Editor actions produced by event handlers
enum Actions {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    EnterMode(Mode),
    PrintChar(char),
    Backspace,
    NewLine,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
}

    /// Handle key events when in Normal mode
    fn handle_normal_event(ev: Event) -> Option<Actions> {
        match ev {
            Event::Key(key) => match key.code {
                KeyCode::Char('h') => Some(Actions::MoveLeft),
                KeyCode::Char('j') => Some(Actions::MoveDown),
                KeyCode::Char('k') => Some(Actions::MoveUp),
                KeyCode::Char('l') => Some(Actions::MoveRight),
                KeyCode::Char('i') => Some(Actions::EnterMode(Mode::Insert)),
                _ => None,
            },
            _ => None,
        }
    }

    /// Handle key events when in Insert mode
    fn handle_insert_event(ev: Event) -> Option<Actions> {
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

    /// Simple in-memory editor state and rendering
    struct Editor {
        buffer: Vec<String>,
        cx: u16,
        cy: u16,
        mode: Mode,
    }

    impl Editor {
        fn new() -> Self {
            Self {
                buffer: vec![String::new()],
                cx: 0,
                cy: 0,
                mode: Mode::Normal,
            }
        }

        fn handle_event(&self, ev: Event) -> Option<Actions> {
            match self.mode {
                Mode::Normal => handle_normal_event(ev),
                Mode::Insert => handle_insert_event(ev),
            }
        }

        fn apply_action(&mut self, action: Actions) {
            match action {
                Actions::MoveLeft => {
                    if self.cx > 0 {
                        self.cx -= 1;
                    }
                }
                Actions::MoveRight => {
                    let line_len = self.buffer[self.cy as usize].len() as u16;
                    if self.cx < line_len {
                        self.cx += 1;
                    }
                }
                Actions::MoveUp => {
                    if self.cy > 0 {
                        self.cy -= 1;
                        let line_len = self.buffer[self.cy as usize].len() as u16;
                        if self.cx > line_len {
                            self.cx = line_len;
                        }
                    }
                }
                Actions::MoveDown => {
                    if (self.cy as usize) + 1 < self.buffer.len() {
                        self.cy += 1;
                        let line_len = self.buffer[self.cy as usize].len() as u16;
                        if self.cx > line_len {
                            self.cx = line_len;
                        }
                    }
                }
                Actions::EnterMode(m) => self.mode = m,
                Actions::PrintChar(c) => {
                    let line = &mut self.buffer[self.cy as usize];
                    let idx = self.cx as usize;
                    if idx <= line.len() {
                        line.insert(idx, c);
                        self.cx += 1;
                    }
                }
                Actions::Backspace => {
                    if self.cx > 0 {
                        let line = &mut self.buffer[self.cy as usize];
                        line.remove((self.cx - 1) as usize);
                        self.cx -= 1;
                    } else if self.cy > 0 {
                        // join with previous line
                        let cur = self.buffer.remove(self.cy as usize);
                        self.cy -= 1;
                        let prev = &mut self.buffer[self.cy as usize];
                        let prev_len = prev.len() as u16;
                        prev.push_str(&cur);
                        self.cx = prev_len;
                    }
                }
                Actions::NewLine => {
                    let line = &mut self.buffer[self.cy as usize];
                    let tail = line.split_off(self.cx as usize);
                    self.buffer.insert((self.cy + 1) as usize, tail);
                    self.cy += 1;
                    self.cx = 0;
                }
            }
        }

        fn render(&self, stdout: &mut impl Write) -> Result<()> {
            let (w, h) = terminal::size()?;
            // clear and render buffer lines
            stdout.queue(terminal::Clear(terminal::ClearType::All))?;
            for (i, line) in self.buffer.iter().enumerate() {
                if i as u16 >= h.saturating_sub(1) {
                    break;
                }
                stdout.queue(MoveTo(0, i as u16))?;
                stdout.queue(Print(line))?;
            }

            // status line at bottom
            let status = match self.mode {
                Mode::Normal => "-- NORMAL --",
                Mode::Insert => "-- INSERT --",
            };
            let status_y = h.saturating_sub(1);
            stdout.queue(MoveTo(0, status_y))?;
            stdout.queue(Print(status))?;

            // position cursor (clamp to terminal size)
            let cx = self.cx.min(w.saturating_sub(1));
            let cy = self.cy.min(h.saturating_sub(1));
            stdout.queue(MoveTo(cx, cy))?;
            stdout.flush()?;
            Ok(())
        }
    }

    fn main() -> Result<()> {
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;
        stdout.execute(terminal::EnterAlternateScreen)?;

        let mut editor = Editor::new();
        editor.render(&mut stdout)?;

        'outer: loop {
            match read()? {
                Event::Key(key) => {
                    // quit only in normal mode with 'q'
                    if editor.mode == Mode::Normal {
                        if let KeyCode::Char('q') = key.code {
                            break 'outer;
                        }
                    }

                    if let Some(action) = editor.handle_event(Event::Key(key)) {
                        editor.apply_action(action);
                        editor.render(&mut stdout)?;
                    }
                }
                _ => {}
            }
        }

        stdout.execute(terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
