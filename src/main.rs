use std::io::stdout;

use anyhow::Result;
use crossterm::event::{read, Event, KeyCode};
use crossterm::{terminal, ExecutableCommand};

mod editor;
use editor::{Editor, Mode};

fn main() -> Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;

    let mut editor = Editor::new();
    editor.render(&mut stdout)?;

    'outer: loop {
        let ev = read()?;
        match ev {
            Event::Key(key) => {
                if editor.mode == Mode::Normal {
                    if let KeyCode::Char('q') = key.code {
                        break 'outer;
                    }
                }

                if let Some(action) = editor.handle_event(ev) {
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
