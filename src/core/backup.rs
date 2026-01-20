/// Backup file handling
use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct BackupManager {
    /// Maximum number of backups to keep
    max_backups: usize,
}

impl BackupManager {
    /// Create a new BackupManager
    pub fn new() -> Self {
        Self { max_backups: 5 }
    }

    /// Create a timestamped backup of a file
    ///
    /// Returns the path to the created backup file
    pub fn create_backup<P: AsRef<Path>>(&self, file_path: P) -> Result<PathBuf> {
        let path = file_path.as_ref();

        // Get absolute path to avoid path resolution issues
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        };

        if !abs_path.exists() {
            // No need to backup if file doesn't exist
            return Ok(PathBuf::new());
        }

        // Generate timestamp for backup filename
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create backup filename: .gitleaksignore.backup.{timestamp}
        let backup_path = abs_path.with_extension(format!("backup.{}", timestamp));

        // Copy file to backup location
        fs::copy(&abs_path, &backup_path)?;

        // Clean up old backups
        self.cleanup_old_backups(&abs_path)?;

        Ok(backup_path)
    }

    /// Remove old backups, keeping only the last max_backups files
    pub fn cleanup_old_backups<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let path = file_path.as_ref();

        // Get absolute path to avoid path resolution issues
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        };

        let parent = abs_path.parent().unwrap_or_else(|| Path::new("."));
        let filename = abs_path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        // Verify parent directory exists and is readable
        if !parent.exists() {
            // Parent directory doesn't exist, skip cleanup (no backups to clean)
            return Ok(());
        }

        // Find all backup files
        let mut backups = Vec::new();

        // Use pattern matching to handle read_dir errors gracefully
        let dir_entries = match fs::read_dir(parent) {
            Ok(entries) => entries,
            Err(_) => {
                // If we can't read the directory, skip cleanup
                // This is not a critical error - backups just won't be cleaned up
                return Ok(());
            }
        };

        for entry in dir_entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // Skip entries we can't read
            };

            let entry_path = entry.path();

            if let Some(entry_name) = entry_path.file_name().and_then(|s| s.to_str()) {
                // Check if this is a backup file for our target file
                if entry_name.starts_with(filename) && entry_name.contains(".backup.") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            backups.push((entry_path, modified));
                        }
                    }
                }
            }
        }

        // Sort backups by modification time (oldest first)
        backups.sort_by_key(|(_, time)| *time);

        // Remove old backups if we exceed max_backups
        if backups.len() > self.max_backups {
            let to_remove = backups.len() - self.max_backups;
            for (backup_path, _) in backups.iter().take(to_remove) {
                let _ = fs::remove_file(backup_path); // Ignore errors on cleanup
            }
        }

        Ok(())
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        Self::new()
    }
}
