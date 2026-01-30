use gh_jobs_purge::*;

/// Integration tests that verify the complete workflow
/// Note: These tests don't actually call GitHub API to avoid rate limiting during tests

#[test]
fn test_rate_limit_parsing_realistic_response() {
    // Simulate a real GitHub API response
    let json = br#"{
        "remaining": 4999,
        "reset": 1706515200
    }"#;

    let result = parse_rate_limit(json).unwrap();
    assert_eq!(result.remaining, 4999);
    assert_eq!(result.reset, 1706515200);
}

#[test]
fn test_workflow_when_quota_healthy() {
    // Scenario: Plenty of quota remaining (>50)
    let remaining = 1000;
    assert!(!should_hibernate(remaining, 50));
}

#[test]
fn test_workflow_when_quota_low() {
    // Scenario: Low quota, should hibernate
    let remaining = 10;
    assert!(should_hibernate(remaining, 50));
}

#[test]
fn test_workflow_wait_calculation() {
    // Scenario: Calculate wait time correctly
    let reset_time = 1000;
    let current_time = 500;

    let wait_seconds = calculate_wait_seconds(reset_time, current_time);
    assert_eq!(wait_seconds, 500);

    // Should be ~8 minutes
    let wait_minutes = wait_seconds / 60;
    assert_eq!(wait_minutes, 8);
}

#[test]
fn test_empty_run_list_means_completion() {
    // Scenario: No more runs to delete means we're done
    let output = "";
    let runs = parse_run_ids(output).unwrap();
    assert!(runs.is_empty(), "Empty run list should signal completion");
}

#[test]
fn test_large_batch_of_runs() {
    // Scenario: Processing a large batch (300 limit)
    let mut output = String::new();
    for i in 1..=300 {
        output.push_str(&format!("{}\n", i));
    }

    let runs = parse_run_ids(&output).unwrap();
    assert_eq!(runs.len(), 300);
    assert_eq!(runs[0], 1);
    assert_eq!(runs[299], 300);
}

#[test]
fn test_secondary_rate_limit_detection_in_mixed_errors() {
    // Scenario: Mix of different errors, one is secondary rate limit
    let errors = vec![
        anyhow::anyhow!("Network timeout"),
        anyhow::anyhow!("Connection refused"),
        anyhow::anyhow!("API error: secondary rate limit exceeded"),
        anyhow::anyhow!("Unknown error"),
    ];

    assert!(check_for_secondary_rate_limit(&errors));
}

#[test]
fn test_no_secondary_rate_limit_in_normal_errors() {
    // Scenario: Various errors but no secondary rate limit
    let errors = vec![
        anyhow::anyhow!("Network timeout"),
        anyhow::anyhow!("Connection refused"),
        anyhow::anyhow!("Run not found"),
        anyhow::anyhow!("Permission denied"),
    ];

    assert!(!check_for_secondary_rate_limit(&errors));
}

#[test]
fn test_edge_case_exactly_at_threshold() {
    // Scenario: Exactly at the threshold (should NOT hibernate)
    assert!(!should_hibernate(50, 50));
}

#[test]
fn test_edge_case_one_below_threshold() {
    // Scenario: One request below threshold (should hibernate)
    assert!(should_hibernate(49, 50));
}

#[test]
fn test_rate_limit_already_reset() {
    // Scenario: Reset time is in the past
    let reset = 100;
    let current = 500;

    let wait = calculate_wait_seconds(reset, current);
    assert_eq!(wait, 0, "Should not wait if reset time has passed");
}

#[test]
fn test_multiple_batches_simulation() {
    // Scenario: Simulating multiple batches being processed
    let batch1 = "1\n2\n3\n4\n5\n";
    let batch2 = "6\n7\n8\n9\n10\n";
    let batch3 = ""; // Empty means done

    let runs1 = parse_run_ids(batch1).unwrap();
    assert_eq!(runs1.len(), 5);

    let runs2 = parse_run_ids(batch2).unwrap();
    assert_eq!(runs2.len(), 5);

    let runs3 = parse_run_ids(batch3).unwrap();
    assert!(runs3.is_empty());
}

#[test]
fn test_malformed_run_ids_filtered_out() {
    // Scenario: Some invalid data in the response
    let output = "12345\n\nabc123\n67890\n  \n99999\n";
    let runs = parse_run_ids(output).unwrap();

    // Should only get valid integers
    assert_eq!(runs, vec![12345, 67890, 99999]);
}

#[test]
fn test_very_large_run_ids() {
    // Scenario: Run IDs can be very large numbers
    let output = "9999999999\n1234567890123\n";
    let runs = parse_run_ids(output).unwrap();

    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0], 9999999999);
    assert_eq!(runs[1], 1234567890123);
}

#[test]
fn test_rate_limit_json_with_extra_fields() {
    // Scenario: GitHub might return extra fields we don't care about
    let json = br#"{
        "remaining": 100,
        "reset": 1234567890,
        "limit": 5000,
        "used": 4900,
        "resource": "core"
    }"#;

    let result = parse_rate_limit(json).unwrap();
    assert_eq!(result.remaining, 100);
    assert_eq!(result.reset, 1234567890);
}

#[test]
fn test_zero_remaining_quota() {
    // Scenario: Completely exhausted quota
    assert!(should_hibernate(0, 50));
}

#[test]
fn test_hibernation_with_different_thresholds() {
    // Scenario: Different projects might use different thresholds
    assert!(should_hibernate(99, 100));
    assert!(should_hibernate(9, 10));
    assert!(should_hibernate(199, 200));

    assert!(!should_hibernate(100, 100));
    assert!(!should_hibernate(10, 10));
    assert!(!should_hibernate(200, 200));
}

#[test]
fn test_status_filter_with_completed() {
    // Scenario: Using the catch-all 'completed' status
    let result = parse_and_validate_statuses("completed").unwrap();
    assert_eq!(result, vec!["completed"]);
}

#[test]
fn test_status_filter_user_friendly_dashes() {
    // Scenario: User prefers dashes over underscores
    let result = parse_and_validate_statuses("in-progress,action-required,timed-out").unwrap();
    assert_eq!(result, vec!["in_progress", "action_required", "timed_out"]);
}

#[test]
fn test_status_filter_mixed_formats() {
    // Scenario: User mixes dashes and underscores
    let result = parse_and_validate_statuses("in-progress,action_required,timed-out").unwrap();
    assert_eq!(result, vec!["in_progress", "action_required", "timed_out"]);
}

#[test]
fn test_status_filter_practical_cleanup() {
    // Scenario: Cleaning up failed and cancelled runs only
    let result = parse_and_validate_statuses("failure,cancelled").unwrap();
    assert_eq!(result, vec!["failure", "cancelled"]);
}

#[test]
fn test_status_filter_all_safe_conclusions() {
    // Scenario: All safe-to-delete conclusion statuses
    let statuses = "success,failure,cancelled,skipped,neutral,stale,timed-out,action-required";
    let result = parse_and_validate_statuses(statuses).unwrap();
    assert_eq!(result.len(), 8);
}

#[test]
fn test_status_normalization_consistency() {
    // Scenario: Both formats should produce identical results
    let with_dashes = parse_and_validate_statuses("in-progress").unwrap();
    let with_underscores = parse_and_validate_statuses("in_progress").unwrap();
    assert_eq!(with_dashes, with_underscores);
}

#[test]
fn test_invalid_status_provides_helpful_error() {
    // Scenario: User makes a typo
    let result = parse_and_validate_statuses("succes"); // Missing 's'
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Invalid status"));
    assert!(err.contains("succes"));
}

#[test]
fn test_status_filter_empty_string() {
    // Scenario: Empty status should error
    let result = parse_and_validate_statuses("");
    assert!(result.is_err());
}

#[test]
fn test_status_filter_whitespace_handling() {
    // Scenario: Extra whitespace should be trimmed
    let result = parse_and_validate_statuses("  success  ,  failure  ").unwrap();
    assert_eq!(result, vec!["success", "failure"]);
}
