use std::path::PathBuf;

/// Persisted SSH hosts file at ~/.config/cmux/ssh_hosts.toml.
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct SshHostsFile {
    #[serde(default)]
    pub hosts: Vec<String>,
}

/// Returns the path to the SSH hosts file.
/// Respects $XDG_CONFIG_HOME/cmux/ssh_hosts.toml; falls back to ~/.config/cmux/ssh_hosts.toml.
pub fn ssh_hosts_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{home}/.config")
    });
    PathBuf::from(base).join("cmux").join("ssh_hosts.toml")
}

/// Parse SSH config text content and extract Host aliases.
/// Filters out wildcard patterns containing `*` or `?`.
pub fn parse_ssh_config_hosts(content: &str) -> Vec<String> {
    let mut hosts = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Host ")
            .or_else(|| trimmed.strip_prefix("Host\t"))
        {
            for host in rest.split_whitespace() {
                if !host.contains('*') && !host.contains('?') {
                    hosts.push(host.to_string());
                }
            }
        }
    }
    hosts
}

/// Load host aliases from ~/.ssh/config. Returns empty vec on any error.
pub fn load_ssh_config_hosts() -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let path = PathBuf::from(home).join(".ssh").join("config");
    match std::fs::read_to_string(&path) {
        Ok(content) => parse_ssh_config_hosts(&content),
        Err(_) => Vec::new(),
    }
}

/// Load saved hosts from ssh_hosts.toml. Returns default on any error.
pub fn load_saved_hosts() -> SshHostsFile {
    let path = ssh_hosts_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => SshHostsFile::default(),
    }
}

/// Save a host target to ssh_hosts.toml. Skips duplicates. Logs errors to stderr.
pub fn save_host(target: &str) {
    let mut file = load_saved_hosts();
    if file.hosts.iter().any(|h| h == target) {
        return; // Already saved
    }
    file.hosts.push(target.to_string());

    let path = ssh_hosts_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("cmux: failed to create config dir: {e}");
            return;
        }
    }

    let content = match toml::to_string_pretty(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cmux: failed to serialize ssh_hosts: {e}");
            return;
        }
    };

    // Atomic write: write to .tmp then rename
    let tmp_path = path.with_extension("toml.tmp");
    if let Err(e) = std::fs::write(&tmp_path, &content) {
        eprintln!("cmux: failed to write ssh_hosts tmp: {e}");
        return;
    }
    if let Err(e) = std::fs::rename(&tmp_path, &path) {
        eprintln!("cmux: failed to rename ssh_hosts tmp: {e}");
    }
}

/// Merge saved hosts and SSH config hosts, deduplicate, return sorted.
pub fn all_hosts() -> Vec<String> {
    let mut hosts = load_saved_hosts().hosts;
    for h in load_ssh_config_hosts() {
        if !hosts.contains(&h) {
            hosts.push(h);
        }
    }
    hosts.sort();
    hosts.dedup();
    hosts
}

/// Extract the hostname from an SSH target string.
/// "user@host.example.com" -> "host.example.com"
/// "host.example.com" -> "host.example.com"
/// "user@host:2222" -> "host"
/// "dev-server" -> "dev-server"
pub fn workspace_name_from_target(target: &str) -> String {
    let host_part = target.split('@').last().unwrap_or(target);
    let without_port = host_part.split(':').next().unwrap_or(host_part);
    without_port.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        assert_eq!(parse_ssh_config_hosts(""), Vec::<String>::new());
    }

    #[test]
    fn test_parse_multi_host_line() {
        let input = "Host foo bar\nHost baz\n";
        assert_eq!(parse_ssh_config_hosts(input), vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_parse_wildcard_filtered() {
        let input = "Host *\nHost dev-?server\nHost good\n";
        assert_eq!(parse_ssh_config_hosts(input), vec!["good"]);
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let input = "  Host   spaced  \n";
        assert_eq!(parse_ssh_config_hosts(input), vec!["spaced"]);
    }

    #[test]
    fn test_parse_hostname_ignored() {
        let input = "HostName not-a-host\nHost real\n";
        assert_eq!(parse_ssh_config_hosts(input), vec!["real"]);
    }

    #[test]
    fn test_round_trip() {
        let file = SshHostsFile {
            hosts: vec!["a".to_string(), "b".to_string()],
        };
        let serialized = toml::to_string_pretty(&file).unwrap();
        let deserialized: SshHostsFile = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.hosts, vec!["a", "b"]);
    }

    #[test]
    fn test_empty_file_default() {
        let file: SshHostsFile = toml::from_str("").unwrap();
        assert!(file.hosts.is_empty());
    }

    #[test]
    fn test_workspace_name_user_at_host() {
        assert_eq!(workspace_name_from_target("user@host.example.com"), "host.example.com");
    }

    #[test]
    fn test_workspace_name_host_only() {
        assert_eq!(workspace_name_from_target("host.example.com"), "host.example.com");
    }

    #[test]
    fn test_workspace_name_with_port() {
        assert_eq!(workspace_name_from_target("user@host:2222"), "host");
    }

    #[test]
    fn test_workspace_name_alias() {
        assert_eq!(workspace_name_from_target("dev-server"), "dev-server");
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join(format!("cmux-ssh-hosts-test-{}", std::process::id()));
        let cfg_dir = dir.join("cmux");
        std::fs::create_dir_all(&cfg_dir).unwrap();

        std::env::set_var("XDG_CONFIG_HOME", dir.to_str().unwrap());

        save_host("user@example.com");
        save_host("another@host");
        // Duplicate should be skipped
        save_host("user@example.com");

        let loaded = load_saved_hosts();
        assert_eq!(loaded.hosts, vec!["user@example.com", "another@host"]);

        std::env::remove_var("XDG_CONFIG_HOME");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
