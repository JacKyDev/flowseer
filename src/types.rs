#[derive(Debug, Clone)]
pub struct AppMetadata {
    pub package_name: &'static str,
    pub version: &'static str,
    pub bin_name: &'static str,
}

impl AppMetadata {
    pub fn from_bin(bin_name: &'static str) -> Self {
        Self {
            package_name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
            bin_name,
        }
    }

    pub fn user_agent(&self) -> String {
        format!(
            "{package}/{version}-{command} ({os}; {arch})",
            package = self.package_name,
            version = self.version,
            command = self.bin_name,
            os = std::env::consts::OS,
            arch = std::env::consts::ARCH,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_agent_contains_package_version_binname() {
        let meta = AppMetadata {
            package_name: "flowseer",
            version: "0.0.1",
            bin_name: "wfgrep",
        };

        let ua = meta.user_agent();

        assert!(ua.starts_with("flowseer/0.0.1-wfgrep"));
    }

    #[test]
    fn user_agent_contains_os_and_arch() {
        let meta = AppMetadata {
            package_name: "flowseer",
            version: "0.0.1",
            bin_name: "wfgrep",
        };

        let ua = meta.user_agent();
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        assert!(ua.contains(&format!("({}; {})", os, arch)));
    }

    #[test]
    fn user_agent_full_format() {
        let meta = AppMetadata {
            package_name: "myapp",
            version: "1.2.3",
            bin_name: "mycmd",
        };

        let expected = format!(
            "myapp/1.2.3-mycmd ({}; {})",
            std::env::consts::OS,
            std::env::consts::ARCH
        );

        assert_eq!(meta.user_agent(), expected);
    }
}
