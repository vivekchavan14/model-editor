use std::io::{Write, stdout};

use crossterm::cursor::MoveTo;
use crossterm::event;
use crossterm::style;
use crossterm::{ExecutableCommand, QueueableCommand, event::read, terminal};

enum Actions {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    EnterMode(Mode),
    PrintChar(char),
    Backspace,
}

enum Mode {
    Normal,
    Insert,
}

impl Mode {
    fn handle_event(&self, ev: event::Event) -> anyhow::Result<Option<Actions>> {
        match self {
            Mode::Normal => Ok(handle_normal_event(ev)),
            Mode::Insert => Ok(handle_insert_event(ev))
        }
    }
}

fn handle_normal_event(ev: event::Event) -> Option<Actions> {
    match ev {
        event::Event::Key(key) => match key.code {
            event::KeyCode::Char('h') => Some(Actions::MoveLeft),
            event::KeyCode::Char('j') => Some(Actions::MoveDown),
            event::KeyCode::Char('k') => Some(Actions::MoveUp),
            event::KeyCode::Char('l') => Some(Actions::MoveRight),
                event::KeyCode::Char('i') => Some(Actions::EnterMode(Mode::Insert)),
            _ => None
        },
        _ => None
    }
}

fn handle_insert_event(ev: event::Event) -> Option<Actions> {
    match ev {
        event::Event::Key(key) => match key.code {
            event::KeyCode::Esc => Some(Actions::EnterMode(Mode::Normal)),
            event::KeyCode::Char(c) => Some(Actions::PrintChar(c)),
            event::KeyCode::Backspace => Some(Actions::Backspace),
            _ => None
        },
        _ => None
    }
}

fn main() -> anyhow::Result<()> {
    let mut stdout = stdout();
    let mut mode = Mode::Normal;
    let mut cx: u16 = 0;
    let mut cy: u16 = 0;
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    loop {
        stdout.queue(MoveTo(cx, cy))?;
        stdout.flush()?;
        
        match read()? {
            event::Event::Key(key) => {
                if let event::KeyCode::Char('q') = key.code {
                    break;
                }
                if let Some(action) = mode.handle_event(event::Event::Key(key))? {
                    match action {
                        Actions::MoveLeft if cx > 0 => cx -= 1,
                        Actions::MoveRight if cx < 79 => cx += 1,
                        Actions::MoveUp if cy > 0 => cy -= 1,
                        Actions::MoveDown if cy < 24 => cy += 1,
                        Actions::EnterMode(new_mode) => mode = new_mode,
                        Actions::PrintChar(c) => {
                            stdout.queue(style::Print(c))?;
                            if cx < 79 { cx += 1; }
                        }
                        Actions::Backspace => {
                            if cx > 0 {
                                cx -= 1;
                                stdout.queue(MoveTo(cx, cy))?;
                                stdout.queue(style::Print(' '))?;
                                stdout.queue(MoveTo(cx, cy))?;
                            }
                        }
                        _ => ()
                    }
                    stdout.flush()?;
                }
            }
            _ => ()
        }
    }
  
    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
