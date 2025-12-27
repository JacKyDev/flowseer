use crate::profile::Profile;
use crate::util::json::deserialize_from_json;
use crate::util::read_file_to_string;
use anyhow::{Context, Result, anyhow};
use std::path::Path;

pub fn load_profile(name: &str, home: &Path) -> Result<Profile> {
    if name.trim().is_empty() {
        return Err(anyhow!("Profile name must not be empty"));
    }

    let mut path = home.to_path_buf();
    path.push(".flowseer");
    path.push(format!("{}.json", name));

    let content =
        read_file_to_string(&path).with_context(|| format!("Failed to load profile {}", name))?;

    let profile: Profile = deserialize_from_json(&content)
        .with_context(|| format!("Failed to parse '{}'\nPath: {}", name, path.display()))?;

    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::fs;
    use tempfile::TempDir;

    fn setup_temp_home() -> TempDir {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let flowseer_dir = tmp.path().join(".flowseer");
        fs::create_dir_all(&flowseer_dir).expect("Failed to create .flowseer dir");
        tmp
    }

    #[test]
    fn rejects_empty_profile_name() {
        let tmp = setup_temp_home();
        let err = load_profile("", tmp.path()).unwrap_err();
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn fails_if_profile_file_missing() {
        let tmp = setup_temp_home();
        let err = load_profile("missing", tmp.path()).unwrap_err();
        assert!(err.to_string().contains("Failed to load profile"));
    }

    #[test]
    fn loads_valid_profile() -> Result<()> {
        let tmp = setup_temp_home();
        let profile_path = tmp.path().join(".flowseer").join("test.json");

        // Beispiel-JSON für Profile
        fs::write(
            &profile_path,
            r#"
            {
                "repo": "my-repo",
                "workflow": "ci.yml",
                "owner": "myorg",
                "token": "ghp_test123",
                "concurrency": 15
            }
            "#,
        )?;

        let profile = load_profile("test", tmp.path())?;

        assert_eq!(profile.repo.as_deref(), Some("my-repo"));
        assert_eq!(profile.workflow.as_deref(), Some("ci.yml"));
        assert_eq!(profile.owner.as_deref(), Some("myorg"));
        assert_eq!(profile.token.as_deref(), Some("ghp_test123"));
        assert_eq!(profile.concurrency, Some(15));

        Ok(())
    }

    #[test]
    fn fails_with_invalid_json() {
        let tmp = setup_temp_home();
        let profile_path = tmp.path().join(".flowseer").join("bad.json");

        fs::write(&profile_path, "{ invalid json").unwrap();

        let err = load_profile("bad", tmp.path()).unwrap_err();
        assert!(err.to_string().contains("Failed to parse"));
    }

    #[test]
    fn fails_with_unknown_fields() {
        let tmp = setup_temp_home();
        let profile_path = tmp.path().join(".flowseer").join("bad.json");

        fs::write(
            &profile_path,
            r#"
            {
                "repo": "my-repo",
                "unknown_field": "oops"
            }
            "#,
        )
        .unwrap();

        let err = load_profile("bad", tmp.path()).unwrap_err();
        assert!(err.to_string().contains("Failed to parse"));
    }
}
