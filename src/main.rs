use std::io::stdout;

use anyhow::Ok;
use crossterm::{ExecutableCommand, terminal};

fn main() -> anyhow::Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    Ok(())
}
