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

    /// Create a new BackupManager with custom max backups
    pub fn with_max_backups(max_backups: usize) -> Self {
        Self { max_backups }
    }

    /// Create a timestamped backup of a file
    ///
    /// Returns the path to the created backup file
    pub fn create_backup<P: AsRef<Path>>(&self, file_path: P) -> Result<PathBuf> {
        let path = file_path.as_ref();

        if !path.exists() {
            // No need to backup if file doesn't exist
            return Ok(PathBuf::new());
        }

        // Generate timestamp for backup filename
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create backup filename: .gitleaksignore.backup.{timestamp}
        let backup_path = path.with_extension(format!("backup.{}", timestamp));

        // Copy file to backup location
        fs::copy(path, &backup_path)?;

        // Clean up old backups
        self.cleanup_old_backups(path)?;

        Ok(backup_path)
    }

    /// Remove old backups, keeping only the last max_backups files
    pub fn cleanup_old_backups<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let path = file_path.as_ref();
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        // Find all backup files
        let mut backups = Vec::new();
        for entry in fs::read_dir(parent)? {
            let entry = entry?;
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

    /// List all backups for a file
    pub fn list_backups<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<PathBuf>> {
        let path = file_path.as_ref();
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        let mut backups = Vec::new();
        for entry in fs::read_dir(parent)? {
            let entry = entry?;
            let entry_path = entry.path();

            if let Some(entry_name) = entry_path.file_name().and_then(|s| s.to_str()) {
                if entry_name.starts_with(filename) && entry_name.contains(".backup.") {
                    backups.push(entry_path);
                }
            }
        }

        Ok(backups)
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        Self::new()
    }
}
