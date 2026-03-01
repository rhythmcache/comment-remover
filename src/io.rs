//! File and directory I/O operations for the comment remover.
//!
//! This module provides essential utilities for interacting with the file system
//! during comment removal. It includes functions for reading and writing files,
//! ensuring parent directories exist, and collecting source files from one or
//! more input paths (with optional recursive directory traversal).
//!
//! All functions in this module return [`Result`] types from the crate's error
//! module, making error handling consistent throughout the application.
//!
//! # Design Philosophy
//!
//! - **Simplicity**: Each function does one thing and does it well.
//! - **Error clarity**: Errors are wrapped in [`AppError::Io`] or specific
//!   variants like [`AppError::DirectoryNotAllowed`] to provide meaningful
//!   context.
//! - **UTF‑8 assumption**: Source code is assumed to be valid UTF‑8. If a file
//!   contains invalid UTF‑8, reading it will fail with an I/O error (the
//!   underlying [`std::fs::read_to_string`] returns an error in that case).
//! - **Symbolic links**: When collecting files, symbolic links are followed
//!   (using [`WalkDir::follow_links(true)`]). This is usually desired when
//!   processing source trees.
//!
//! # Typical Workflow
//!
//! 1. Use [`collect_files`] to obtain a list of all files to process from
//!    user‑provided paths (with optional recursion).
//! 2. For each file, call [`read_file`] to get its content.
//! 3. Process the content (remove comments, collapse whitespace).
//! 4. Determine the output destination:
//!    - If writing back to the original file, use [`write_file`] directly.
//!    - If writing to an output directory, compute the new path and call
//!      [`create_parent_dir`] to ensure the directory exists, then [`write_file`].
//!
//! # Example
//!
//! ```
//! use comment_remover::io::{collect_files, read_file, write_file, create_parent_dir};
//! use std::path::PathBuf;
//!
//! # fn example() -> comment_remover::error::Result<()> {
//! let input_paths = vec![PathBuf::from("src")];
//! let recursive = true;
//! let files = collect_files(&input_paths, recursive)?;
//!
//! for file in files {
//!     let content = read_file(&file)?;
//!     // ... process content (remove comments) ...
//!     let output_path = PathBuf::from("out").join(file);
//!     create_parent_dir(&output_path)?;
//!     write_file(&output_path, &content)?;
//! }
//! # Ok(())
//! # }
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::error::{AppError, Result};

/// Reads the entire contents of a file into a string.
///
/// This function is a thin wrapper around [`std::fs::read_to_string`] that
/// converts the I/O error into [`AppError::Io`].
///
/// # Arguments
///
/// * `path` – The path to the file to read.
///
/// # Returns
///
/// The contents of the file as a `String`.
///
/// # Errors
///
/// Returns [`AppError::Io`] in the following situations:
/// - The file does not exist.
/// - The process lacks permission to read the file.
/// - The file is not valid UTF‑8.
/// - Any other I/O error that [`std::fs::read_to_string`] may return.
///
/// # Example
///
/// ```
/// # use comment_remover::io::read_file;
/// # use std::path::Path;
/// # fn example() -> comment_remover::error::Result<()> {
/// let content = read_file(Path::new("Cargo.toml"))?;
/// println!("Cargo.toml is {} bytes", content.len());
/// # Ok(())
/// # }
/// ```
pub fn read_file(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(AppError::Io)
}

/// Writes a string to a file, overwriting any existing content.
///
/// This function wraps [`std::fs::write`] and converts its errors into
/// [`AppError::Io`]. It does **not** create missing parent directories;
/// you should call [`create_parent_dir`] beforehand if the target path
/// may have non‑existent parent components.
///
/// # Arguments
///
/// * `path` – The path where the content should be written.
/// * `content` – The string content to write.
///
/// # Returns
///
/// `Ok(())` on success.
///
/// # Errors
///
/// Returns [`AppError::Io`] if:
/// - The parent directory does not exist.
/// - The process lacks write permission.
/// - Any other I/O error occurs.
///
/// # Example
///
/// ```
/// # use comment_remover::io::write_file;
/// # use std::path::Path;
/// # use tempfile::NamedTempFile;
/// # fn example() -> comment_remover::error::Result<()> {
/// # let tmp = NamedTempFile::new().unwrap();
/// write_file(tmp.path(), "cleaned source code")?;
/// # Ok(())
/// # }
/// ```
pub fn write_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(AppError::Io)
}

/// Ensures that the parent directory of a path exists.
///
/// If the path has a parent directory (e.g., `"out/sub/file.txt"` has parent
/// `"out/sub"`), this function creates that directory and all its ancestors
/// using [`fs::create_dir_all`]. If the parent already exists, it does nothing.
/// If the path has no parent (e.g., a file in the current directory like
/// `"file.txt"`), it does nothing.
///
/// # Arguments
///
/// * `path` – The path whose parent directory should be created.
///
/// # Returns
///
/// `Ok(())` on success.
///
/// # Errors
///
/// Returns [`AppError::Io`] if the directory cannot be created (e.g., due to
/// permission issues or a file with the same name already exists in the path).
///
/// # Example
///
/// ```
/// # use comment_remover::io::create_parent_dir;
/// # use std::path::Path;
/// # use tempfile::tempdir;
/// # fn example() -> comment_remover::error::Result<()> {
/// let dir = tempdir()?;
/// let deep_path = dir.path().join("a/b/c/file.txt");
/// create_parent_dir(&deep_path)?; // creates a/, a/b/, a/b/c/
/// assert!(deep_path.parent().unwrap().exists());
/// # Ok(())
/// # }
/// ```
pub fn create_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
    }
    Ok(())
}

/// Collects all files from a list of input paths.
///
/// This function expands a list of input paths into a flat list of regular
/// files suitable for processing. The behaviour depends on whether `recursive`
/// is enabled:
///
/// - If a path points to a regular file, it is always included.
/// - If a path points to a directory and `recursive` is `true`, all files
///   inside that directory (including subdirectories) are included.
/// - If a path points to a directory and `recursive` is `false`, an error
///   [`AppError::DirectoryNotAllowed`] is returned.
/// - If a path does not exist, an error [`AppError::Io`] (with
///   `ErrorKind::NotFound`) is returned.
///
/// Symbolic links are followed: if a link points to a file or directory, it
/// is treated as the target. The function silently skips non‑file, non‑directory
/// entries (e.g., sockets, devices) during recursive traversal.
///
/// # Arguments
///
/// * `paths` – A slice of [`PathBuf`] entries to process.
/// * `recursive` – Whether to recursively traverse directories.
///
/// # Returns
///
/// A vector of [`PathBuf`] containing all regular files found, in no particular
/// order. The order is not guaranteed and may vary between calls.
///
/// # Errors
///
/// - Returns [`AppError::DirectoryNotAllowed`] if a directory is encountered
///   when `recursive` is `false`.
/// - Returns [`AppError::Io`] if any path does not exist, or if an I/O error
///   occurs while querying metadata (e.g., permission denied on a directory).
///
/// # Example
///
/// ```
/// # use comment_remover::io::collect_files;
/// # use std::path::PathBuf;
/// # use tempfile::{tempdir, NamedTempFile};
/// # use std::fs::File;
/// # fn example() -> comment_remover::error::Result<()> {
/// let dir = tempdir()?;
/// let file1 = dir.path().join("a.rs");
/// File::create(&file1)?;
/// let subdir = dir.path().join("sub");
/// std::fs::create_dir(&subdir)?;
/// let file2 = subdir.join("b.rs");
/// File::create(&file2)?;
///
/// let paths = vec![dir.path().to_path_buf()];
/// let files = collect_files(&paths, true)?;
/// assert_eq!(files.len(), 2);
/// assert!(files.contains(&file1));
/// assert!(files.contains(&file2));
/// # Ok(())
/// # }
/// ```
pub fn collect_files(paths: &[PathBuf], recursive: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for path in paths {
        if !path.exists() {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Path does not exist: {}", path.display()),
            )));
        }
        // Helper to get metadata with proper error conversion
        pub fn get_metadata(path: &Path) -> Result<fs::Metadata> {
            fs::metadata(path).map_err(AppError::Io)
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
            // Skip entries we cannot read (permission denied, etc.)
            {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    files.push(entry_path.to_path_buf());
                }
            }
        } else {
            // Not a file or directory (e.g., socket, device) – skip silently.
            // This matches the typical expectation of ignoring special files.
        }
    }

    Ok(files)
}
