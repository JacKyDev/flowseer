use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

pub fn read_file_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn reads_existing_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        fs::write(&file_path, "hello world").unwrap();

        let content = read_file_to_string(&file_path).unwrap();

        assert_eq!(content, "hello world");
    }

    #[test]
    fn fails_with_context_when_file_does_not_exist() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("missing.txt");

        let err = read_file_to_string(&file_path).unwrap_err();

        let msg = err.to_string();
        assert!(msg.contains("Failed to read file"));
        assert!(msg.contains("missing.txt"));
    }
}
