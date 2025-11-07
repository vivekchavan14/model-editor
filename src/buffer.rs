use std::io;
use thiserror::Error;

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
}

impl Buffer {
    pub fn from_file(file: Option<String>) -> Result<Self, BufferError> {
        let lines = match &file {
            Some(file_path) => {
                if !std::path::Path::new(file_path).exists() {
                    return Err(BufferError::FileNotFound(file_path.clone()));
                }
                std::fs::read_to_string(file_path)?
                    .lines()
                    .map(|s| s.to_string())
                    .collect()
            }
            None => vec![String::new()],
        };
        Ok(Self { file, lines })
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
        let line_content = self.get_line_mut(line)?;
        if col > line_content.len() {
            return Err(BufferError::InvalidColumnIndex(col, line));
        }
        line_content.insert(col, c);
        Ok(())
    }

    pub fn remove_char(&mut self, line: usize, col: usize) -> Result<char, BufferError> {
        let line_content = self.get_line_mut(line)?;
        if col >= line_content.len() {
            return Err(BufferError::InvalidColumnIndex(col, line));
        }
        Ok(line_content.remove(col))
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
        let previous_line = self.get_line_mut(line_index - 1)?;
        let previous_length = previous_line.len();
        previous_line.push_str(&current_line);
        
        Ok(previous_length)
    }

    pub fn save(&self) -> Result<(), BufferError> {
        let file_path = self.file.as_ref()
            .ok_or_else(|| BufferError::FileNotFound("No file path set".to_string()))?;
        
        let content = self.lines.join("\n");
        std::fs::write(file_path, content)?;
        Ok(())
    }

    pub fn save_as(&mut self, file_path: String) -> Result<(), BufferError> {
        if std::path::Path::new(&file_path).exists() {
            std::fs::write(&file_path, self.lines.join("\n"))?;
            self.file = Some(file_path);
            Ok(())
        } else {
            let parent = std::path::Path::new(&file_path)
                .parent()
                .ok_or_else(|| BufferError::FileNotFound("Invalid path".to_string()))?;
            
            std::fs::create_dir_all(parent)?;
            std::fs::write(&file_path, self.lines.join("\n"))?;
            self.file = Some(file_path);
            Ok(())
        }
    }
}
