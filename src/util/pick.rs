use anyhow::anyhow;

pub fn pick_required<T>(cli: Option<T>, profile: Option<T>, name: &str) -> anyhow::Result<T> {
    cli.or(profile).ok_or_else(|| {
        anyhow!(
            "Missing required parameter: {}\n\
             Hint: Provide it via CLI or define it in the selected profile",
            name
        )
    })
}

pub fn pick_vec<T>(cli: Vec<T>, profile: Option<Vec<T>>) -> Vec<T> {
    if !cli.is_empty() {
        cli
    } else {
        profile.unwrap_or_default()
    }
}

pub fn pick_default<T>(cli: T, profile: Option<T>) -> T {
    profile.unwrap_or(cli)
}

pub fn pick_optional<T>(cli: Option<T>, profile: Option<T>) -> Option<T> {
    cli.or(profile)
}

pub fn pick_bool(cli: bool, profile: Option<bool>) -> bool {
    if cli { true } else { profile.unwrap_or(false) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_value_wins_over_profile() {
        let result =
            pick_required(Some("cli".to_string()), Some("profile".to_string()), "repo").unwrap();

        assert_eq!(result, "cli");
    }

    #[test]
    fn profile_value_used_when_cli_missing() {
        let result = pick_required::<String>(None, Some("profile".to_string()), "repo").unwrap();

        assert_eq!(result, "profile");
    }

    #[test]
    fn fails_when_neither_cli_nor_profile_provided() {
        let err = pick_required::<String>(None, None, "repo").unwrap_err();

        let msg = err.to_string();
        assert!(msg.contains("Missing required parameter: repo"));
        assert!(msg.contains("Provide it via CLI or define it in the selected profile"));
    }

    #[test]
    fn works_with_non_string_types() {
        let result = pick_required(Some(42u8), None, "concurrency").unwrap();
        assert_eq!(result, 42);
    }
}

#[cfg(test)]
mod pick_vec_tests {
    use super::*;

    #[test]
    fn cli_vec_wins_when_not_empty() {
        let result = pick_vec(vec![1, 2, 3], Some(vec![9, 9]));
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn profile_vec_used_when_cli_empty() {
        let result = pick_vec::<i32>(vec![], Some(vec![4, 5]));
        assert_eq!(result, vec![4, 5]);
    }

    #[test]
    fn empty_when_cli_and_profile_missing() {
        let result = pick_vec::<i32>(vec![], None);
        assert!(result.is_empty());
    }
}

#[cfg(test)]
mod pick_default_tests {
    use super::*;

    #[test]
    fn profile_value_overrides_cli() {
        let result = pick_default(5u8, Some(10));
        assert_eq!(result, 10);
    }

    #[test]
    fn cli_used_when_profile_missing() {
        let result = pick_default(5u8, None);
        assert_eq!(result, 5);
    }

    #[test]
    fn works_with_non_numeric_types() {
        let result = pick_default("cli".to_string(), Some("profile".to_string()));
        assert_eq!(result, "profile");
    }
}

#[cfg(test)]
mod pick_bool_tests {
    use super::*;

    #[test]
    fn cli_true_always_wins() {
        let result = pick_bool(true, Some(false));
        assert!(result);
    }

    #[test]
    fn profile_true_used_when_cli_false() {
        let result = pick_bool(false, Some(true));
        assert!(result);
    }

    #[test]
    fn false_when_both_cli_and_profile_false() {
        let result = pick_bool(false, Some(false));
        assert!(!result);
    }

    #[test]
    fn false_when_neither_cli_nor_profile_true() {
        let result = pick_bool(false, None);
        assert!(!result);
    }
}

#[cfg(test)]
mod pick_optional_tests {
    use super::*;

    #[test]
    fn cli_wins_when_present() {
        let result = pick_optional(Some(5), Some(10));
        assert_eq!(result, Some(5));
    }

    #[test]
    fn profile_used_when_cli_missing() {
        let result = pick_optional::<i32>(None, Some(10));
        assert_eq!(result, Some(10));
    }

    #[test]
    fn none_when_both_missing() {
        let result = pick_optional::<i32>(None, None);
        assert_eq!(result, None);
    }
}
