const MAX_STREAM_BYTES: usize = 24 * 1024;
const MAX_COMBINED_BYTES: usize = 32 * 1024;
const MAX_LINES: usize = 400;
const MAX_LINE_LENGTH: usize = 1_000;
const OUTPUT_OMITTED: &str = "output_omitted";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellOutputSummary {
    pub summary: String,
    pub output_omitted: bool,
    pub labels: Vec<String>,
}

pub struct ShellOutputSummarizer;

impl ShellOutputSummarizer {
    pub fn summarize(stdout: &[u8], stderr: &[u8]) -> ShellOutputSummary {
        let mut labels = Vec::new();

        if looks_binary(stdout) || looks_binary(stderr) {
            return omitted("output_omitted_binary_or_control_data");
        }

        let Ok(stdout_text) = std::str::from_utf8(stdout) else {
            return omitted("output_omitted_untrusted_encoding");
        };
        let Ok(stderr_text) = std::str::from_utf8(stderr) else {
            return omitted("output_omitted_untrusted_encoding");
        };

        let redacted_stdout = redact(stdout_text, &mut labels);
        let redacted_stderr = redact(stderr_text, &mut labels);
        let mut combined = format!("stdout:\n{redacted_stdout}\nstderr:\n{redacted_stderr}");

        combined = truncate_lines(&combined, &mut labels);

        if combined.len() > MAX_COMBINED_BYTES {
            combined.truncate(MAX_COMBINED_BYTES);
            labels.push("truncated_by_size".into());
        }

        ShellOutputSummary {
            summary: combined,
            output_omitted: false,
            labels: dedupe(labels),
        }
    }
}

fn redact(input: &str, labels: &mut Vec<String>) -> String {
    let mut output = input.to_string();
    let patterns = [
        "sk-or-",
        "sk-",
        "ghp_",
        "gho_",
        "ghu_",
        "ghs_",
        "github_pat_",
        "Bearer ",
    ];

    for pattern in patterns {
        while let Some(index) = output.find(pattern) {
            let end = output[index..]
                .find(|character: char| character.is_whitespace())
                .map(|offset| index + offset)
                .unwrap_or(output.len());
            output.replace_range(index..end, "[redacted]");
            labels.push("redacted_secret_pattern".into());
        }
    }

    if output.contains("-----BEGIN") && output.contains("PRIVATE KEY-----") {
        output = "[redacted private key]".into();
        labels.push("redacted_private_key".into());
    }

    output
}

fn truncate_lines(input: &str, labels: &mut Vec<String>) -> String {
    let mut lines = Vec::new();

    for line in input.lines().take(MAX_LINES) {
        if line.len() > MAX_LINE_LENGTH {
            lines.push(line.chars().take(MAX_LINE_LENGTH).collect::<String>());
            labels.push("truncated_by_line_length".into());
        } else {
            lines.push(line.to_string());
        }
    }

    if input.lines().count() > MAX_LINES {
        labels.push("truncated_by_line_count".into());
    }

    let mut output = lines.join("\n");

    if output.len() > MAX_STREAM_BYTES * 2 {
        output.truncate(MAX_STREAM_BYTES * 2);
        labels.push("truncated_by_size".into());
    }

    output
}

fn looks_binary(bytes: &[u8]) -> bool {
    bytes.iter().any(|byte| *byte == 0)
}

fn omitted(label: &str) -> ShellOutputSummary {
    ShellOutputSummary {
        summary: OUTPUT_OMITTED.into(),
        output_omitted: true,
        labels: vec![label.into()],
    }
}

fn dedupe(labels: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();

    for label in labels {
        if !deduped.contains(&label) {
            deduped.push(label);
        }
    }

    deduped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_secret_shaped_values() {
        let summary =
            ShellOutputSummarizer::summarize(b"token sk-or-secret-value\n", b"github ghp_secret\n");

        assert!(!summary.summary.contains("sk-or-secret-value"));
        assert!(!summary.summary.contains("ghp_secret"));
        assert!(summary.labels.contains(&"redacted_secret_pattern".into()));
    }

    #[test]
    fn truncates_large_output_with_safe_label() {
        let output = "line\n".repeat(600);

        let summary = ShellOutputSummarizer::summarize(output.as_bytes(), b"");

        assert!(summary.summary.lines().count() <= MAX_LINES);
        assert!(summary.labels.contains(&"truncated_by_line_count".into()));
    }

    #[test]
    fn omits_binary_output_without_raw_fallback() {
        let summary = ShellOutputSummarizer::summarize(b"hello\0secret", b"");

        assert_eq!(summary.summary, OUTPUT_OMITTED);
        assert!(summary.output_omitted);
        assert_eq!(
            summary.labels,
            vec!["output_omitted_binary_or_control_data"]
        );
    }

    #[test]
    fn omits_untrusted_encoding_without_raw_fallback() {
        let summary = ShellOutputSummarizer::summarize(&[0xff, 0xfe], b"");

        assert_eq!(summary.summary, OUTPUT_OMITTED);
        assert!(summary.output_omitted);
        assert_eq!(summary.labels, vec!["output_omitted_untrusted_encoding"]);
    }

    #[test]
    fn redacts_private_key_as_omitted_content() {
        let summary = ShellOutputSummarizer::summarize(
            b"-----BEGIN OPENSSH PRIVATE KEY-----\nabc\n-----END OPENSSH PRIVATE KEY-----",
            b"",
        );

        assert!(!summary.summary.contains("abc"));
        assert!(summary.labels.contains(&"redacted_private_key".into()));
    }
}
