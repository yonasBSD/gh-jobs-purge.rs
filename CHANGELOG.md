# Changelog

## Version 0.2.0 - Status Filtering Feature

### Added
- **`--status` CLI flag** using clap 4 for filtering runs by status
- Support for all GitHub Actions statuses:
  - Runtime: `queued`, `in-progress`, `requested`, `waiting`, `pending`
  - Conclusion: `success`, `failure`, `cancelled`, `skipped`, `neutral`, `stale`, `timed-out`, `action-required`
  - Catch-all: `completed`
- **Dash/underscore normalization**: Accept both `in-progress` and `in_progress`
- **Comma-separated lists**: Filter by multiple statuses at once
- **Status validation**: Helpful error messages for invalid statuses
- **19 new tests** for status filtering functionality (66 total tests now)
- `EXAMPLES.md` - Comprehensive usage examples
- `fetch_runs_with_statuses()` - Multi-status fetching with deduplication

### Changed
- `fetch_completed_runs()` now calls `fetch_runs_with_statuses()` internally
- Main loop now shows which statuses are being filtered
- Success message includes the status filter applied
- README updated with status filtering documentation
- Test count increased from 47 to 66 tests

### Technical Details
- Added `clap` dependency for CLI argument parsing
- Status validation happens before any GitHub API calls
- Multiple status filters result in multiple API calls (one per status)
- Run IDs are deduplicated automatically when using multiple statuses
- All status names are normalized (dashes → underscores) before API calls

## Version 0.1.0 - Initial Release

### Features
- Delete completed GitHub Actions workflow runs
- Pre-flight quota checking
- Smart hibernation when rate limit low (<50)
- Parallel deletion (15 threads via Rayon)
- Secondary rate limit detection with 60s backoff
- Colorful emoji-rich terminal output
- 47 automated tests with 100% coverage of core logic
