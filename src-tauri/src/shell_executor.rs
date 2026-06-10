use crate::path_resolver::ProjectPathResolver;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Eq, PartialEq)]
pub struct ShellExecutionResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ShellExecutionError {
    OutsideProjectRoot,
    SpawnFailed(String),
}

pub struct ShellExecutor {
    project_root: std::path::PathBuf,
}

impl ShellExecutor {
    pub fn new(project_root: impl AsRef<Path>) -> Result<Self, ShellExecutionError> {
        let resolver = ProjectPathResolver::new(project_root)
            .map_err(|_| ShellExecutionError::OutsideProjectRoot)?;
        let root = resolver
            .resolve_existing(
                ".",
                crate::path_resolver::FileAccessActor::Browser,
                crate::path_resolver::FileAccessOperation::Read,
            )
            .map_err(|_| ShellExecutionError::OutsideProjectRoot)?
            .project_root;

        Ok(Self { project_root: root })
    }

    pub fn execute(
        &self,
        command: &str,
        working_directory: impl AsRef<Path>,
        env: &HashMap<String, String>,
    ) -> Result<ShellExecutionResult, ShellExecutionError> {
        let working_directory = std::fs::canonicalize(working_directory)
            .map_err(|_| ShellExecutionError::OutsideProjectRoot)?;

        if !working_directory.starts_with(&self.project_root) {
            return Err(ShellExecutionError::OutsideProjectRoot);
        }

        let mut filtered_env = filter_environment(env);
        filtered_env.insert(
            "PWD".into(),
            working_directory.to_string_lossy().to_string(),
        );

        let output = Command::new("/bin/sh")
            .arg("-c")
            .arg(command)
            .current_dir(working_directory)
            .env_clear()
            .envs(filtered_env)
            .output()
            .map_err(|error| ShellExecutionError::SpawnFailed(error.to_string()))?;

        Ok(ShellExecutionResult {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

pub fn filter_environment(env: &HashMap<String, String>) -> HashMap<String, String> {
    let mut filtered = HashMap::new();

    for (key, value) in env {
        if should_keep_env(key, value) {
            filtered.insert(key.clone(), value.clone());
        }
    }

    filtered
}

fn should_keep_env(key: &str, value: &str) -> bool {
    let upper = key.to_uppercase();

    if is_secret_env_name(&upper) || looks_secret_value(value) {
        return false;
    }

    matches!(
        upper.as_str(),
        "PATH"
            | "HOME"
            | "USER"
            | "LOGNAME"
            | "SHELL"
            | "PWD"
            | "LANG"
            | "TERM"
            | "TMPDIR"
            | "TMP"
            | "TEMP"
            | "CI"
            | "NO_COLOR"
            | "FORCE_COLOR"
    ) || upper.starts_with("LC_")
        || upper.starts_with("CARGO_")
        || upper.starts_with("RUSTUP_")
        || upper.starts_with("NPM_CONFIG_")
        || upper.starts_with("YARN_")
        || upper.starts_with("PNPM_")
}

fn is_secret_env_name(upper: &str) -> bool {
    upper.ends_with("_KEY")
        || upper.ends_with("_TOKEN")
        || upper.ends_with("_SECRET")
        || upper.ends_with("_PASSWORD")
        || upper.ends_with("_PASS")
        || upper.ends_with("_PWD")
        || upper.contains("_CREDENTIAL")
        || upper.contains("_AUTH")
        || upper.contains("_SESSION")
        || upper.contains("_COOKIE")
        || upper.contains("_BEARER")
        || upper.contains("_PRIVATE")
        || upper.starts_with("OPENAI_")
        || upper.starts_with("OPENROUTER_")
        || upper.starts_with("ANTHROPIC_")
        || upper == "GOOGLE_API_KEY"
        || upper.starts_with("GEMINI_")
        || upper == "AWS_ACCESS_KEY_ID"
        || upper == "AWS_SECRET_ACCESS_KEY"
        || upper == "AWS_SESSION_TOKEN"
        || upper == "AWS_PROFILE"
        || upper.starts_with("AZURE_")
        || upper.starts_with("GCP_")
        || upper == "GOOGLE_APPLICATION_CREDENTIALS"
        || upper == "GH_TOKEN"
        || upper == "GITHUB_TOKEN"
        || upper == "GIT_ASKPASS"
        || upper == "SSH_ASKPASS"
        || upper == "NPM_TOKEN"
        || upper == "NODE_AUTH_TOKEN"
        || upper == "PYPI_TOKEN"
        || upper == "TWINE_PASSWORD"
        || upper == "KUBECONFIG"
}

fn looks_secret_value(value: &str) -> bool {
    value.starts_with("sk-")
        || value.starts_with("sk-or-")
        || value.starts_with("ghp_")
        || value.starts_with("github_pat_")
        || value.contains("-----BEGIN") && value.contains("PRIVATE KEY-----")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn environment_filter_strips_secret_names_and_values() {
        let env = HashMap::from([
            ("PATH".into(), "/usr/bin:/bin".into()),
            ("OPENROUTER_API_KEY".into(), "sk-or-secret".into()),
            ("CUSTOM".into(), "sk-secret-value".into()),
            ("LANG".into(), "en_US.UTF-8".into()),
        ]);

        let filtered = filter_environment(&env);

        assert!(filtered.contains_key("PATH"));
        assert!(filtered.contains_key("LANG"));
        assert!(!filtered.contains_key("OPENROUTER_API_KEY"));
        assert!(!filtered.contains_key("CUSTOM"));
    }

    #[test]
    fn shell_executes_in_project_working_directory() {
        let directory = tempdir().expect("tempdir");
        let executor = ShellExecutor::new(directory.path()).expect("executor");
        let env = HashMap::from([("PATH".into(), "/usr/bin:/bin".into())]);

        let result = executor
            .execute("pwd", directory.path(), &env)
            .expect("command runs");

        assert_eq!(result.exit_code, Some(0));
        assert_eq!(
            std::fs::canonicalize(result.stdout.trim()).expect("stdout path canonical"),
            directory
                .path()
                .canonicalize()
                .expect("temp path canonical")
        );
    }

    #[test]
    fn shell_blocks_outside_root_working_directory() {
        let directory = tempdir().expect("tempdir");
        let outside = tempdir().expect("outside");
        let executor = ShellExecutor::new(directory.path()).expect("executor");
        let env = HashMap::new();

        let result = executor.execute("pwd", outside.path(), &env);

        assert_eq!(result, Err(ShellExecutionError::OutsideProjectRoot));
    }
}
