use std::io::{self, stdout};
use std::panic;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use crossterm::event::{read, Event, KeyCode};
use crossterm::{terminal, ExecutableCommand};
use log::{debug, error, info, warn};
use dirs::home_dir;

mod editor;
use editor::{Editor, Mode};

mod buffer;
mod logger;

static PANIC_CLEANUP: AtomicBool = AtomicBool::new(false);

fn cleanup() -> io::Result<()> {
    if !PANIC_CLEANUP.swap(true, Ordering::SeqCst) {
        debug!("Performing terminal cleanup");
        terminal::disable_raw_mode()?;
        stdout().execute(terminal::LeaveAlternateScreen)?;
        info!("Terminal cleanup completed");
    } else {
        warn!("Cleanup already performed, skipping");
    }
    Ok(())
}

fn main() -> Result<()> {
    // Initialize logger with log file in user's home directory
    let log_path = home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".vix")
        .join("vix.log");
    
    logger::FileLogger::init(log_path)?;
    info!("Starting vix editor");
    
    let mut stdout = stdout();
    debug!("Initializing terminal in raw mode");
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;

    let file = std::env::args().nth(1);
    debug!("Opening file: {:?}", file);
    let buffer = buffer::Buffer::from_file(file)?;
    let mut editor = Editor::with_buffer(buffer);
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        error!("Panic occurred: {}", panic_info);
        if let Err(e) = cleanup() {
            error!("Error during cleanup: {}", e);
        }
        original_hook(panic_info);
        process::exit(1);
    }));

    editor.render(&mut stdout)?;

    'outer: loop {
        let ev = read()?;
        match ev {
            Event::Key(key) => {
                debug!("Key event received: {:?}", key);
                if editor.mode == Mode::Normal {
                    if let KeyCode::Char('q') = key.code {
                        info!("Quit command received, exiting editor");
                        break 'outer;
                    }
                }

                if let Some(action) = editor.handle_event(ev) {
                    debug!("Applying editor action");
                    editor.apply_action(action);
                    editor.render(&mut stdout)?;
                }
            }
            _ => {
                debug!("Non-key event received: {:?}", ev);
            }
        }
    }

    cleanup()?;
    Ok(())
}
