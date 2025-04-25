/// Functionality to clean up stale data from a given directory.
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use anyhow::Result;

/// Cleans up stale data from the specified directory.
/// It removes all files and directories that are older than the specified age.
///
/// # Arguments
/// * `dir` - The directory to clean up.
/// * `age` - The age threshold in as Duration. Files and directories older than this will be removed.
///
/// # Returns
/// A Result indicating success or failure.
pub(crate) fn cleanup_stale_data(dir: &Path, age: &Duration) -> Result<()> {
    // Check if the directory exists
    if !dir.exists() {
        return Err(anyhow::anyhow!(
            "Directory does not exist: {}",
            dir.display()
        ));
    }

    // Get the current time
    let now = std::time::SystemTime::now();

    // Recursively inspect the directory and remove stale data
    inspect_dir(dir, age, &now)?;

    Ok(())
}

fn inspect_dir(dir: &Path, age: &Duration, now: &SystemTime) -> Result<(bool)> {
    // if .keep is in the directory, do not remove anything
    if dir.join(".keep").exists() {
        return Ok(false);
    }

    let mut all_removed = true;

    for entry in dir.read_dir()? {
        let path = entry?.path();
        if path.is_dir() {
            all_removed &= inspect_dir(&path, age, now)?;
        } else {
            let metadata = path.metadata()?;
            let modified = metadata.modified()?;
            let duration = now.duration_since(modified)?;

            if duration > *age {
                fs::remove_file(&path)?;
                all_removed &= true;
            } else {
                all_removed &= false;
            }
        }
    }

    if all_removed {
        fs::remove_dir(dir)?;
    }

    Ok(all_removed)
}
