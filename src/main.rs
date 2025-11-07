use std::io::{self, stdout, Write};
use std::panic;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use crossterm::event::{read, Event, KeyCode};
use crossterm::{terminal, ExecutableCommand};

mod editor;
use editor::{Editor, Mode};

mod buffer;

static PANIC_CLEANUP: AtomicBool = AtomicBool::new(false);

fn cleanup() -> io::Result<()> {
    if !PANIC_CLEANUP.swap(true, Ordering::SeqCst) {
        terminal::disable_raw_mode()?;
        stdout().execute(terminal::LeaveAlternateScreen)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;

    let file = std::env::args().nth(1);
    let buffer = buffer::Buffer::from_file(file)?;
    let mut editor = Editor::with_buffer(buffer);
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        if let Err(e) = cleanup() {
            eprintln!("Error during cleanup: {}", e);
        }
        eprintln!("Panic occurred: {}", panic_info);
        original_hook(panic_info);
        process::exit(1);
    }));

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

    cleanup()?;
    Ok(())
}
