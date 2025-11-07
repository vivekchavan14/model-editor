use std::io;
use thiserror::Error;
use log::{debug, error, info, warn};

#[derive(Error, Debug)]
pub enum BufferError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Invalid line index: {0}")]
    InvalidLineIndex(usize),
    #[error("Invalid column index: {0} in line {1}")]
    InvalidColumnIndex(usize, usize),
}

pub struct Buffer {
    pub file: Option<String>,
    pub lines: Vec<String>,
    pub modified: bool,
}

impl Buffer {
    pub fn from_file(file: Option<String>) -> Result<Self, BufferError> {
        let lines = match &file {
            Some(file_path) => {
                info!("Opening file: {}", file_path);
                if !std::path::Path::new(file_path).exists() {
                    warn!("File not found: {}", file_path);
                    return Err(BufferError::FileNotFound(file_path.clone()));
                }
                let content: Vec<String> = std::fs::read_to_string(file_path)?
                    .lines()
                    .map(|s| s.to_string())
                    .collect();
                debug!("Read {} lines from file", content.len());
                content
            }
            None => {
                info!("Creating new empty buffer");
                vec![String::new()]
            }
        };
        Ok(Self { file, lines, modified: false })
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn get_line(&self, index: usize) -> Result<&String, BufferError> {
        self.lines.get(index)
            .ok_or(BufferError::InvalidLineIndex(index))
    }

    pub fn get_line_mut(&mut self, index: usize) -> Result<&mut String, BufferError> {
        self.lines.get_mut(index)
            .ok_or(BufferError::InvalidLineIndex(index))
    }

    pub fn insert_char(&mut self, line: usize, col: usize, c: char) -> Result<(), BufferError> {
        {
            let line_content = self.get_line_mut(line)?;
            if col > line_content.len() {
                return Err(BufferError::InvalidColumnIndex(col, line));
            }
            line_content.insert(col, c);
        }
        self.modified = true;
        Ok(())
    }

    pub fn remove_char(&mut self, line: usize, col: usize) -> Result<char, BufferError> {
        let removed = {
            let line_content = self.get_line_mut(line)?;
            if col >= line_content.len() {
                return Err(BufferError::InvalidColumnIndex(col, line));
            }
            line_content.remove(col)
        };
        self.modified = true;
        Ok(removed)
    }

    pub fn line_length(&self, index: usize) -> Result<usize, BufferError> {
        self.get_line(index).map(|line| line.len())
    }

    pub fn display_name(&self) -> String {
        match &self.file {
            Some(path) => path.clone(),
            None => "[No Name]".to_string(),
        }
    }

    pub fn join_with_previous_line(&mut self, line_index: usize) -> Result<usize, BufferError> {
        if line_index == 0 {
            return Err(BufferError::InvalidLineIndex(line_index));
        }

        let current_line = self.lines.remove(line_index);
        let previous_length = {
            let previous_line = self.get_line_mut(line_index - 1)?;
            let len = previous_line.len();
            previous_line.push_str(&current_line);
            len
        };
        self.modified = true;
        Ok(previous_length)
    }

    pub fn delete_line(&mut self, index: usize) -> Result<(), BufferError> {
        if self.lines.is_empty() {
            return Err(BufferError::InvalidLineIndex(index));
        }
        if self.lines.len() == 1 {
            // keep a single empty line
            self.lines[0].clear();
            self.modified = true;
            return Ok(());
        }
        if index >= self.lines.len() {
            return Err(BufferError::InvalidLineIndex(index));
        }
        self.lines.remove(index);
        self.modified = true;
        Ok(())
    }

    pub fn save(&self) -> Result<(), BufferError> {
        let file_path = self.file.as_ref()
            .ok_or_else(|| BufferError::FileNotFound("No file path set".to_string()))?;
        
        let content = self.lines.join("\n");
        std::fs::write(file_path, &content)?;
        debug!("Successfully saved {} bytes to {}", content.len(), file_path);
        Ok(())
    }

    pub fn save_as(&mut self, file_path: String) -> Result<(), BufferError> {
        info!("Saving as: {}", file_path);
        if std::path::Path::new(&file_path).exists() {
            debug!("File exists, overwriting");
            let content = self.lines.join("\n");
            std::fs::write(&file_path, &content)?;
            debug!("Successfully saved {} bytes", content.len());
            self.file = Some(file_path);
            self.modified = false;
            Ok(())
        } else {
            let parent = std::path::Path::new(&file_path)
                .parent()
                .ok_or_else(|| {
                    warn!("Invalid path provided for save_as");
                    BufferError::FileNotFound("Invalid path".to_string())
                })?;
            
            debug!("Creating directory structure: {:?}", parent);
            std::fs::create_dir_all(parent)?;
            let content = self.lines.join("\n");
            std::fs::write(&file_path, &content)?;
            debug!("Successfully saved {} bytes", content.len());
            self.file = Some(file_path);
            self.modified = false;
            Ok(())
        }
    }

    /// Attempts to save any modified changes to a recovery file during a panic
    pub fn try_save_recovery(&self) {
        if !self.modified {
            debug!("Buffer not modified, skipping recovery save");
            return;
        }

        let recovery_path = match &self.file {
            Some(path) => format!("{}.recovery", path),
            None => ".unnamed.recovery".to_string(),
        };

        let content = self.lines.join("\n");
        if let Err(e) = std::fs::write(&recovery_path, &content) {
            error!("Failed to save recovery file: {}", e);
        } else {
            debug!("Recovery file saved: {} ({} bytes)", recovery_path, content.len());
        }
    }
}
