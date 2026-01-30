use std::process::Command;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct RateLimitCore {
    pub remaining: i32,
    pub reset: i64,
}

/// Valid runtime statuses (active runs)
const RUNTIME_STATUSES: &[&str] = &["queued", "in_progress", "requested", "waiting", "pending"];

/// Valid conclusion statuses (finished runs)
const CONCLUSION_STATUSES: &[&str] = &[
    "success",
    "failure",
    "cancelled",
    "skipped",
    "neutral",
    "stale",
    "timed_out",
    "action_required",
];

/// The catch-all status
const COMPLETED_STATUS: &str = "completed";

/// Normalize a status by replacing dashes with underscores
pub fn normalize_status(status: &str) -> String {
    status.replace('-', "_")
}

/// Validate and normalize a comma-separated list of statuses
pub fn parse_and_validate_statuses(input: &str) -> Result<Vec<String>> {
    let statuses: Vec<String> = input
        .split(',')
        .map(|s| normalize_status(s.trim()))
        .collect();

    for status in &statuses {
        if !is_valid_status(status) {
            anyhow::bail!(
                "Invalid status '{}'. Valid statuses are:\n\
                 Runtime: {}\n\
                 Conclusion: {}\n\
                 Catch-all: completed",
                status,
                RUNTIME_STATUSES.join(", "),
                CONCLUSION_STATUSES.join(", ")
            );
        }
    }

    Ok(statuses)
}

/// Check if a status is valid
pub fn is_valid_status(status: &str) -> bool {
    status == COMPLETED_STATUS
        || RUNTIME_STATUSES.contains(&status)
        || CONCLUSION_STATUSES.contains(&status)
}

/// Parse rate limit JSON response
pub fn parse_rate_limit(json_data: &[u8]) -> Result<RateLimitCore> {
    serde_json::from_slice(json_data).context("Failed to parse rate limit JSON")
}

/// Parse run IDs from gh CLI output
pub fn parse_run_ids(output: &str) -> Result<Vec<i64>> {
    let runs: Vec<i64> = output
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.parse().ok())
        .collect();

    Ok(runs)
}

/// Check if any error indicates a secondary rate limit was hit
pub fn check_for_secondary_rate_limit(errors: &[anyhow::Error]) -> bool {
    errors.iter().any(|e| {
        e.to_string()
            .to_lowercase()
            .contains("secondary rate limit")
    })
}

/// Calculate wait time until rate limit reset
pub fn calculate_wait_seconds(reset_timestamp: i64, current_time: i64) -> i64 {
    (reset_timestamp - current_time).max(0)
}

/// Determine if we should hibernate based on remaining quota
pub fn should_hibernate(remaining: i32, threshold: i32) -> bool {
    remaining < threshold
}

/// Check GitHub API rate limit status
pub fn check_rate_limit() -> Result<RateLimitCore> {
    let output = Command::new("gh")
        .args(["api", "rate_limit", "--jq", ".resources.core"])
        .output()
        .context("Failed to execute gh api rate_limit")?;

    if !output.status.success() {
        anyhow::bail!("gh api rate_limit command failed");
    }

    parse_rate_limit(&output.stdout)
}

/// Fetch completed GitHub Action run IDs
pub fn fetch_completed_runs() -> Result<Vec<i64>> {
    fetch_runs_with_statuses(&["completed".to_string()])
}

/// Fetch GitHub Action run IDs filtered by status
pub fn fetch_runs_with_statuses(statuses: &[String]) -> Result<Vec<i64>> {
    let mut all_runs = Vec::new();

    for status in statuses {
        let output = Command::new("gh")
            .args([
                "run",
                "list",
                "--status",
                status,
                "--limit",
                "300",
                "--json",
                "databaseId",
                "-q",
                ".[].databaseId",
            ])
            .output()
            .context(format!(
                "Failed to execute gh run list for status '{}'",
                status
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh run list failed for status '{}': {}", status, stderr);
        }

        let runs = parse_run_ids(&String::from_utf8_lossy(&output.stdout))?;
        all_runs.extend(runs);
    }

    // Remove duplicates (in case a run matches multiple statuses, though unlikely)
    all_runs.sort_unstable();
    all_runs.dedup();

    Ok(all_runs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rate_limit_valid_json() {
        let json = br#"{"remaining":100,"reset":1234567890}"#;
        let result = parse_rate_limit(json).unwrap();
        assert_eq!(result.remaining, 100);
        assert_eq!(result.reset, 1234567890);
    }

    #[test]
    fn test_parse_rate_limit_invalid_json() {
        let json = b"not json";
        assert!(parse_rate_limit(json).is_err());
    }

    #[test]
    fn test_parse_rate_limit_missing_fields() {
        let json = br#"{"remaining":100}"#;
        assert!(parse_rate_limit(json).is_err());
    }

    #[test]
    fn test_parse_run_ids_empty() {
        let output = "";
        let result = parse_run_ids(output).unwrap();
        assert_eq!(result, Vec::<i64>::new());
    }

    #[test]
    fn test_parse_run_ids_single() {
        let output = "12345\n";
        let result = parse_run_ids(output).unwrap();
        assert_eq!(result, vec![12345]);
    }

    #[test]
    fn test_parse_run_ids_multiple() {
        let output = "12345\n67890\n11111\n";
        let result = parse_run_ids(output).unwrap();
        assert_eq!(result, vec![12345, 67890, 11111]);
    }

    #[test]
    fn test_parse_run_ids_with_empty_lines() {
        let output = "12345\n\n67890\n\n";
        let result = parse_run_ids(output).unwrap();
        assert_eq!(result, vec![12345, 67890]);
    }

    #[test]
    fn test_parse_run_ids_invalid_numbers() {
        let output = "12345\nabc\n67890\n";
        let result = parse_run_ids(output).unwrap();
        // Invalid lines are filtered out
        assert_eq!(result, vec![12345, 67890]);
    }

    #[test]
    fn test_parse_run_ids_negative_numbers() {
        let output = "12345\n-67890\n11111\n";
        let result = parse_run_ids(output).unwrap();
        // Negative numbers are valid i64
        assert_eq!(result, vec![12345, -67890, 11111]);
    }

    #[test]
    fn test_check_for_secondary_rate_limit_empty() {
        let errors: Vec<anyhow::Error> = vec![];
        assert!(!check_for_secondary_rate_limit(&errors));
    }

    #[test]
    fn test_check_for_secondary_rate_limit_no_match() {
        let errors = vec![
            anyhow::anyhow!("Some other error"),
            anyhow::anyhow!("Network timeout"),
        ];
        assert!(!check_for_secondary_rate_limit(&errors));
    }

    #[test]
    fn test_check_for_secondary_rate_limit_match_lowercase() {
        let errors = vec![
            anyhow::anyhow!("Some other error"),
            anyhow::anyhow!("Hit secondary rate limit"),
        ];
        assert!(check_for_secondary_rate_limit(&errors));
    }

    #[test]
    fn test_check_for_secondary_rate_limit_match_uppercase() {
        let errors = vec![anyhow::anyhow!("SECONDARY RATE LIMIT exceeded")];
        assert!(check_for_secondary_rate_limit(&errors));
    }

    #[test]
    fn test_check_for_secondary_rate_limit_match_mixed_case() {
        let errors = vec![anyhow::anyhow!("Error: Secondary Rate Limit reached")];
        assert!(check_for_secondary_rate_limit(&errors));
    }

    #[test]
    fn test_calculate_wait_seconds_future() {
        let reset = 1000;
        let current = 500;
        assert_eq!(calculate_wait_seconds(reset, current), 500);
    }

    #[test]
    fn test_calculate_wait_seconds_past() {
        let reset = 500;
        let current = 1000;
        // Should return 0, not negative
        assert_eq!(calculate_wait_seconds(reset, current), 0);
    }

    #[test]
    fn test_calculate_wait_seconds_same_time() {
        let reset = 1000;
        let current = 1000;
        assert_eq!(calculate_wait_seconds(reset, current), 0);
    }

    #[test]
    fn test_calculate_wait_seconds_large_difference() {
        let reset = i64::MAX;
        let current = 0;
        assert_eq!(calculate_wait_seconds(reset, current), i64::MAX);
    }

    #[test]
    fn test_should_hibernate_below_threshold() {
        assert!(should_hibernate(49, 50));
        assert!(should_hibernate(0, 50));
        assert!(should_hibernate(1, 50));
    }

    #[test]
    fn test_should_hibernate_at_threshold() {
        assert!(!should_hibernate(50, 50));
    }

    #[test]
    fn test_should_hibernate_above_threshold() {
        assert!(!should_hibernate(51, 50));
        assert!(!should_hibernate(100, 50));
        assert!(!should_hibernate(5000, 50));
    }

    #[test]
    fn test_should_hibernate_custom_threshold() {
        assert!(should_hibernate(99, 100));
        assert!(!should_hibernate(100, 100));
        assert!(!should_hibernate(101, 100));
    }

    #[test]
    fn test_should_hibernate_negative_remaining() {
        // Edge case: negative remaining (shouldn't happen but handle gracefully)
        assert!(should_hibernate(-1, 50));
        assert!(should_hibernate(-100, 50));
    }

    #[test]
    fn test_normalize_status_with_dashes() {
        assert_eq!(normalize_status("in-progress"), "in_progress");
        assert_eq!(normalize_status("timed-out"), "timed_out");
        assert_eq!(normalize_status("action-required"), "action_required");
    }

    #[test]
    fn test_normalize_status_without_dashes() {
        assert_eq!(normalize_status("completed"), "completed");
        assert_eq!(normalize_status("success"), "success");
        assert_eq!(normalize_status("queued"), "queued");
    }

    #[test]
    fn test_is_valid_status_runtime() {
        assert!(is_valid_status("queued"));
        assert!(is_valid_status("in_progress"));
        assert!(is_valid_status("requested"));
        assert!(is_valid_status("waiting"));
        assert!(is_valid_status("pending"));
    }

    #[test]
    fn test_is_valid_status_conclusion() {
        assert!(is_valid_status("success"));
        assert!(is_valid_status("failure"));
        assert!(is_valid_status("cancelled"));
        assert!(is_valid_status("skipped"));
        assert!(is_valid_status("neutral"));
        assert!(is_valid_status("stale"));
        assert!(is_valid_status("timed_out"));
        assert!(is_valid_status("action_required"));
    }

    #[test]
    fn test_is_valid_status_completed() {
        assert!(is_valid_status("completed"));
    }

    #[test]
    fn test_is_valid_status_invalid() {
        assert!(!is_valid_status("invalid"));
        assert!(!is_valid_status("running"));
        assert!(!is_valid_status(""));
        assert!(!is_valid_status("COMPLETED"));
    }

    #[test]
    fn test_parse_and_validate_statuses_single() {
        let result = parse_and_validate_statuses("completed").unwrap();
        assert_eq!(result, vec!["completed"]);
    }

    #[test]
    fn test_parse_and_validate_statuses_multiple() {
        let result = parse_and_validate_statuses("success,failure,cancelled").unwrap();
        assert_eq!(result, vec!["success", "failure", "cancelled"]);
    }

    #[test]
    fn test_parse_and_validate_statuses_with_dashes() {
        let result = parse_and_validate_statuses("in-progress,timed-out").unwrap();
        assert_eq!(result, vec!["in_progress", "timed_out"]);
    }

    #[test]
    fn test_parse_and_validate_statuses_with_spaces() {
        let result = parse_and_validate_statuses("success, failure, cancelled").unwrap();
        assert_eq!(result, vec!["success", "failure", "cancelled"]);
    }

    #[test]
    fn test_parse_and_validate_statuses_mixed() {
        let result = parse_and_validate_statuses("completed,queued,success,in-progress").unwrap();
        assert_eq!(result, vec![
            "completed",
            "queued",
            "success",
            "in_progress"
        ]);
    }

    #[test]
    fn test_parse_and_validate_statuses_invalid() {
        assert!(parse_and_validate_statuses("invalid").is_err());
        assert!(parse_and_validate_statuses("success,invalid,failure").is_err());
        assert!(parse_and_validate_statuses("").is_err());
    }

    #[test]
    fn test_parse_and_validate_statuses_all_runtime() {
        let result =
            parse_and_validate_statuses("queued,in-progress,requested,waiting,pending").unwrap();
        assert_eq!(result, vec![
            "queued",
            "in_progress",
            "requested",
            "waiting",
            "pending"
        ]);
    }

    #[test]
    fn test_parse_and_validate_statuses_all_conclusion() {
        let result = parse_and_validate_statuses(
            "success,failure,cancelled,skipped,neutral,stale,timed-out,action-required",
        )
        .unwrap();
        assert_eq!(result, vec![
            "success",
            "failure",
            "cancelled",
            "skipped",
            "neutral",
            "stale",
            "timed_out",
            "action_required"
        ]);
    }
}
