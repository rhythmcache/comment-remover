//! File and directory I/O utilities for the comment remover.
//!
//! This module provides helper functions for common file system operations:
//! reading files, writing files, creating parent directories, and collecting
//! input files from paths (including recursive directory traversal).
//!
//! All functions return [`Result`] with [`AppError`], providing detailed
//! error context including the path that caused the error.
//!
//! # Examples
//!
//! Basic usage:
//! ```
//! use comment_remover::io;
//! use std::path::PathBuf;
//!
//! // Collect all Rust files recursively from a directory
//! let paths = vec![PathBuf::from("src")];
//! let files = io::collect_files(&paths, true).unwrap();
//!
//! for file in files {
//!     let content = io::read_file(&file).unwrap();
//!     println!("{}: {} bytes", file.display(), content.len());
//! }
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::error::{AppError, Result, io_error};

/// Retrieves metadata for a file or directory.
///
/// This is a private helper that wraps `std::fs::metadata` and converts I/O errors
/// into [`AppError::Io`] with the associated path.
///
/// # Arguments
///
/// * `path` - Path to the file or directory.
///
/// # Returns
///
/// * `Ok(fs::Metadata)` on success.
/// * `Err(AppError::Io)` if the metadata cannot be read.
fn get_metadata(path: &Path) -> Result<fs::Metadata> {
    fs::metadata(path).map_err(|e| io_error(path, e))
}

/// Reads the entire contents of a file into a string.
///
/// This function is a thin wrapper around `std::fs::read_to_string` that converts
/// I/O errors into [`AppError::Io`] with the file path.
///
/// # Arguments
///
/// * `path` - Path to the file to read.
///
/// # Returns
///
/// * `Ok(String)` containing the file contents.
/// * `Err(AppError::Io)` if the file cannot be read (e.g., not found, permission denied).
///
/// # Examples
///
/// ```
/// use comment_remover::io;
/// use std::path::Path;
///
/// let content = io::read_file(Path::new("Cargo.toml")).unwrap();
/// println!("Cargo.toml:\n{}", content);
/// ```
pub fn read_file(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|e| io_error(path, e))
}

/// Writes a string to a file, overwriting any existing content.
///
/// This function wraps `std::fs::write` and converts I/O errors into [`AppError::Io`].
/// It does **not** automatically create parent directories; call [`create_parent_dir`]
/// first if needed.
///
/// # Arguments
///
/// * `path` - Path where the file will be written.
/// * `content` - The string content to write.
///
/// # Returns
///
/// * `Ok(())` on successful write.
/// * `Err(AppError::Io)` if the write fails (e.g., permission denied, disk full).
///
/// # Examples
///
/// ```
/// use comment_remover::io;
/// use std::path::Path;
///
/// io::write_file(Path::new("output.txt"), "Hello, world!").unwrap();
/// ```
pub fn write_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|e| io_error(path, e))
}

/// Ensures that the parent directory of a given path exists.
///
/// If the parent directory does not exist, it is created recursively
/// (like `mkdir -p`). If the parent already exists, this function does nothing.
/// This is useful before writing a file to guarantee that the directory structure
/// is in place.
///
/// # Arguments
///
/// * `path` - A path whose parent directory should exist.
///
/// # Returns
///
/// * `Ok(())` if the parent directory exists or was successfully created.
/// * `Err(AppError::Io)` if directory creation fails.
///
/// # Examples
///
/// ```
/// use comment_remover::io;
/// use std::path::Path;
///
/// let out_path = Path::new("out/subdir/file.txt");
/// io::create_parent_dir(out_path).unwrap();
/// io::write_file(out_path, "data").unwrap();
/// ```
pub fn create_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| io_error(parent, e))?;
        }
    }
    Ok(())
}

/// Collects all files from a list of input paths.
///
/// For each path:
/// * If it is a regular file, it is added to the result.
/// * If it is a directory and `recursive` is `true`, all files inside that directory
///   (including subdirectories) are added. Symbolic links are followed.
/// * If it is a directory and `recursive` is `false`, an error [`AppError::DirectoryNotAllowed`]
///   is returned.
///
/// If any input path does not exist, an I/O error is returned immediately.
///
/// # Arguments
///
/// * `paths` - A slice of [`PathBuf`] representing input files or directories.
/// * `recursive` - If `true`, directories are traversed recursively; otherwise,
///   directories are rejected.
///
/// # Returns
///
/// * `Ok(Vec<PathBuf>)` containing all discovered files.
/// * `Err(AppError::Io)` if a path does not exist or cannot be accessed.
/// * `Err(AppError::DirectoryNotAllowed)` if a directory is given without `recursive`.
///
/// # Examples
///
/// Collect all files from current directory recursively:
/// ```
/// use comment_remover::io;
/// use std::path::PathBuf;
///
/// let paths = vec![PathBuf::from(".")];
/// let files = io::collect_files(&paths, true).unwrap();
/// for file in files {
///     println!("Found: {}", file.display());
/// }
/// ```
///
/// Process a single file (non-recursive):
/// ```
/// use comment_remover::io;
/// use std::path::PathBuf;
///
/// let paths = vec![PathBuf::from("src/main.rs")];
/// let files = io::collect_files(&paths, false).unwrap();
/// assert_eq!(files.len(), 1);
/// ```
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
