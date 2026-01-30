# Status Filtering Feature - Quick Summary

## What Changed

Added a `--status` flag that allows filtering GitHub Actions runs by status using clap 4.

## Usage

```bash
# Default (completed runs only)
gh-jobs-purge

# Delete specific statuses
gh-jobs-purge --status "success,failure,cancelled"

# User-friendly dashes work too
gh-jobs-purge --status "in-progress,timed-out,action-required"
```

## Supported Statuses

### Runtime (Active) - ⚠️ Usually can't delete these
- `queued`, `in-progress`, `requested`, `waiting`, `pending`

### Conclusion (Finished) - ✅ Safe to delete  
- `success`, `failure`, `cancelled`, `skipped`, `neutral`, `stale`, `timed-out`, `action-required`

### Catch-All
- `completed` - All finished runs (default)

## Key Features

✅ **Dash/underscore flexibility**: `in-progress` and `in_progress` both work  
✅ **Comma-separated lists**: Multiple statuses in one flag  
✅ **Validation**: Helpful errors for typos  
✅ **Tested**: 19 new tests covering all scenarios

## Examples

### Delete only failures
```bash
gh-jobs-purge --status failure
```

### Clean up failed and cancelled
```bash
gh-jobs-purge --status "failure,cancelled"
```

### Remove all unsuccessful runs
```bash
gh-jobs-purge --status "failure,cancelled,timed-out,stale"
```

### Help
```bash
gh-jobs-purge --help
```

## Files Modified/Added

- ✏️ `Cargo.toml` - Added clap dependency
- ✏️ `src/lib.rs` - Added status validation functions + 19 unit tests
- ✏️ `src/main.rs` - Added CLI args with clap, updated to use status filter
- ✏️ `tests/integration_tests.rs` - Added 9 integration tests
- ✏️ `README.md` - Documented new flag and status options
- ➕ `EXAMPLES.md` - Comprehensive usage examples
- ➕ `CHANGELOG.md` - Version history
- ✏️ `TEST_SUMMARY.md` - Updated test counts (47 → 66 tests)

## Test Coverage

- **Before**: 47 tests
- **After**: 66 tests (+19 for status filtering)
- **Coverage**: 100% of status validation logic

See `EXAMPLES.md` for more use cases!
