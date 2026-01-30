# Usage Examples

## Basic Usage

### Delete all completed runs (default)
```bash
gh-jobs-purge
# or
gh-jobs-purge --status completed
```

## Deleting Specific Conclusion Statuses

### Delete only failed runs
```bash
gh-jobs-purge --status failure
```

### Delete failed and cancelled runs
```bash
gh-jobs-purge --status "failure,cancelled"
```

### Delete all unsuccessful runs (failures, cancellations, timeouts)
```bash
gh-jobs-purge --status "failure,cancelled,timed-out"
```

### Clean up stale and timed-out runs
```bash
gh-jobs-purge --status "stale,timed-out,action-required"
```

### Delete everything except successful runs
```bash
gh-jobs-purge --status "failure,cancelled,skipped,neutral,stale,timed-out,action-required"
```

## Using Dashes vs Underscores

Both formats work identically:

```bash
# With dashes (user-friendly)
gh-jobs-purge --status "in-progress,timed-out,action-required"

# With underscores (GitHub API format)
gh-jobs-purge --status "in_progress,timed_out,action_required"

# Mixed (also works!)
gh-jobs-purge --status "in-progress,timed_out,action-required"
```

## Deleting Active Runs (Advanced)

⚠️ **Warning**: These usually fail with 403 Forbidden errors since GitHub prevents deleting active runs.

### Try to delete queued runs
```bash
gh-jobs-purge --status queued
```

### Try to delete in-progress runs
```bash
gh-jobs-purge --status in-progress
```

### Try to delete all pending/waiting runs
```bash
gh-jobs-purge --status "pending,waiting,requested"
```

## Combining Multiple Statuses

### All conclusion statuses (comprehensive cleanup)
```bash
gh-jobs-purge --status "success,failure,cancelled,skipped,neutral,stale,timed-out,action-required"
```

### Mix of runtime and conclusion (though runtime deletions likely fail)
```bash
gh-jobs-purge --status "queued,failure,cancelled"
```

## Help and Version

### Show help
```bash
gh-jobs-purge --help
```

Output:
```
GitHub Actions workflow run purge tool

Usage: gh-jobs-purge [OPTIONS]

Options:
  -s, --status <STATUS>  Comma-separated list of statuses to filter runs
                         
                         Runtime statuses (active runs):
                           queued, in-progress, requested, waiting, pending
                         
                         Conclusion statuses (finished runs):
                           success, failure, cancelled, skipped, neutral, stale, timed-out, action-required
                         
                         Catch-all:
                           completed (all finished runs)
                         
                         Note: Use dashes (-) or underscores (_) interchangeably (e.g., in-progress or in_progress)
                         
                         [default: completed]
  -h, --help             Print help
  -V, --version          Print version
```

### Show version
```bash
gh-jobs-purge --version
```

## Common Workflows

### Weekly cleanup: Delete all old runs regardless of status
```bash
# Delete everything (successful and failed)
gh-jobs-purge --status completed
```

### Daily cleanup: Only delete failures to keep history of successes
```bash
gh-jobs-purge --status "failure,cancelled,timed-out"
```

### Emergency cleanup: Free up storage by removing everything
```bash
gh-jobs-purge --status "success,failure,cancelled,skipped,neutral,stale,timed-out,action-required"
```

### Conservative cleanup: Only remove obvious failures
```bash
gh-jobs-purge --status "failure,cancelled"
```

## Error Handling

### Invalid status
```bash
gh-jobs-purge --status "invalid-status"
```

Output:
```
Error: Invalid status argument

Caused by:
    Invalid status 'invalid_status'. Valid statuses are:
    Runtime: queued, in_progress, requested, waiting, pending
    Conclusion: success, failure, cancelled, skipped, neutral, stale, timed_out, action_required
    Catch-all: completed
```

### Empty status
```bash
gh-jobs-purge --status ""
```

Output:
```
Error: Invalid status argument
```

## Tips

1. **Start with defaults**: Use `--status completed` to delete all finished runs
2. **Be specific**: Target specific statuses like `failure` to keep successful runs
3. **Use dashes**: They're more user-friendly than underscores
4. **Check before deleting**: The tool shows which statuses it's filtering before starting
5. **Watch rate limits**: The tool automatically handles rate limits and hibernates when needed
