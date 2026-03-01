use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::error::{AppError, Result, io_error};

fn get_metadata(path: &Path) -> Result<fs::Metadata> {
    fs::metadata(path).map_err(|e| io_error(path, e))
}

pub fn read_file(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|e| io_error(path, e))
}

pub fn write_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|e| io_error(path, e))
}

pub fn create_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| io_error(parent, e))?;
        }
    }
    Ok(())
}

pub fn collect_files(paths: &[PathBuf], recursive: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for path in paths {
        if !path.exists() {
            return Err(io_error(
                path,
                std::io::Error::new(std::io::ErrorKind::NotFound, "path does not exist"),
            ));
        }

        let metadata = get_metadata(path)?;
        if metadata.is_file() {
            files.push(path.clone());
        } else if metadata.is_dir() {
            if !recursive {
                return Err(AppError::DirectoryNotAllowed(path.display().to_string()));
            }

            for entry in WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    files.push(entry_path.to_path_buf());
                }
            }
        }
    }

    Ok(files)
}
