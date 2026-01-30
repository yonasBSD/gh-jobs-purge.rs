# Test Quick Reference

## Run Tests

```bash
cargo test                          # All tests
cargo test --lib                    # Unit tests only
cargo test --test integration_tests # Integration tests only
cargo test test_name                # Specific test
cargo test -- --nocapture           # Show println! output
cargo test -- --test-threads=1      # Run tests serially
```

## Test Organization

```
gh-jobs-purge/
├── src/
│   ├── lib.rs              # Library with 27 unit tests
│   └── main.rs             # Binary (manually tested)
├── tests/
│   └── integration_tests.rs # 20 integration tests
└── TESTING.md              # Detailed test docs
```

## Coverage by Component

| Component | Function | Tests | Status |
|-----------|----------|-------|--------|
| **Parsing** | `parse_rate_limit()` | 4 | ✅ |
| | `parse_run_ids()` | 8 | ✅ |
| **Logic** | `check_for_secondary_rate_limit()` | 6 | ✅ |
| | `calculate_wait_seconds()` | 5 | ✅ |
| | `should_hibernate()` | 9 | ✅ |
| **Integration** | Workflow scenarios | 20 | ✅ |

## Example Test Output

```
running 47 tests
test tests::test_calculate_wait_seconds_future ... ok
test tests::test_calculate_wait_seconds_large_difference ... ok
test tests::test_calculate_wait_seconds_past ... ok
test tests::test_calculate_wait_seconds_same_time ... ok
test tests::test_check_for_secondary_rate_limit_empty ... ok
test tests::test_check_for_secondary_rate_limit_match_lowercase ... ok
test tests::test_check_for_secondary_rate_limit_match_mixed_case ... ok
test tests::test_check_for_secondary_rate_limit_match_uppercase ... ok
test tests::test_check_for_secondary_rate_limit_no_match ... ok
test tests::test_parse_rate_limit_invalid_json ... ok
test tests::test_parse_rate_limit_missing_fields ... ok
test tests::test_parse_rate_limit_valid_json ... ok
test tests::test_parse_run_ids_empty ... ok
test tests::test_parse_run_ids_invalid_numbers ... ok
test tests::test_parse_run_ids_multiple ... ok
test tests::test_parse_run_ids_negative_numbers ... ok
test tests::test_parse_run_ids_single ... ok
test tests::test_parse_run_ids_with_empty_lines ... ok
test tests::test_should_hibernate_above_threshold ... ok
test tests::test_should_hibernate_at_threshold ... ok
test tests::test_should_hibernate_below_threshold ... ok
test tests::test_should_hibernate_custom_threshold ... ok
test tests::test_should_hibernate_negative_remaining ... ok
...

test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Key Test Scenarios

### ✅ Happy Path
- Valid JSON parsing
- Normal quota levels (>50)
- Successful run deletion
- Clean termination

### ⚠️ Edge Cases
- Exactly at threshold (50)
- One below threshold (49)
- Empty run list
- Already-reset timestamp

### ❌ Error Cases
- Malformed JSON
- Invalid run IDs
- Secondary rate limit
- Negative/zero quota

### 🔄 Integration
- Multiple batch processing
- Hibernation triggered
- Rate limit detection
- Time calculations

## Writing New Tests

```rust
#[test]
fn test_new_feature() {
    // Arrange
    let input = setup_test_data();
    
    // Act
    let result = function_under_test(input);
    
    // Assert
    assert_eq!(result, expected);
}
```

## Debugging Tests

```bash
# Show output
cargo test -- --nocapture

# Run single test with backtrace
RUST_BACKTRACE=1 cargo test test_name -- --nocapture

# Show ignored tests
cargo test -- --ignored
```
