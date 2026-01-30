# GitHub Jobs Purge

Purge all completed GitHub Action runs.

## Features

✅ **Pre-flight quota checking** - Verifies API rate limit before making requests  
✅ **Smart hibernation** - Sleeps until rate limit reset when quota is low (<50 remaining)  
✅ **Parallel deletion** - Deletes up to 15 runs concurrently using Rayon  
✅ **Secondary rate limit detection** - Automatically backs off when hitting burst limits  
✅ **Colorful output** - Emoji-rich terminal feedback  
✅ **Graceful error handling** - Retries on network issues or API errors

## Prerequisites

- Rust (1.70+)
- `gh` CLI installed and authenticated

## Installation

```bash
cd gh-jobs-purge
cargo build --release
```

## Running Tests

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# See TESTING.md for detailed test documentation
```

## Usage

```bash
# Default: Delete all completed runs
cargo run --release

# Delete specific statuses
cargo run --release -- --status "success,failure"

# Delete queued runs (use with caution!)
cargo run --release -- --status "queued"

# Multiple statuses with dashes or underscores
cargo run --release -- --status "in-progress,timed-out,action-required"
cargo run --release -- --status "in_progress,timed_out,action_required"  # Same thing

# Get help
cargo run --release -- --help
```

Or install it to your PATH:

```bash
cargo install --path .

# Then use it directly
gh-jobs-purge
gh-jobs-purge --status "success,failure,cancelled"
```

## Status Filters

The `--status` flag accepts a comma-separated list of statuses:

### Runtime Statuses (Active Runs)
⚠️ **Warning**: Deleting active runs usually fails with a 403 Forbidden error.
- `queued`
- `in-progress` (or `in_progress`)
- `requested`
- `waiting`
- `pending`

### Conclusion Statuses (Finished Runs)
✅ **Safe to delete**
- `success`
- `failure`
- `cancelled`
- `skipped`
- `neutral`
- `stale`
- `timed-out` (or `timed_out`)
- `action-required` (or `action_required`)

### Catch-All
- `completed` - All finished runs (default)

**Note**: You can use dashes (`-`) or underscores (`_`) interchangeably. Both `in-progress` and `in_progress` work.

## How It Works

1. **Parse Arguments**: Validates and normalizes status filters from `--status` flag
2. **Rate Limit Check**: Queries GitHub API quota before proceeding
3. **Hibernation**: If <50 requests remaining, sleeps until reset time
4. **Fetch Runs**: Gets up to 300 run IDs per status (multiple API calls if needed)
5. **Parallel Delete**: Spawns 15 worker threads to delete runs concurrently
6. **Backoff**: If secondary rate limit hit, waits 60 seconds
7. **Loop**: Continues until no matching runs remain

## Comparison to Fish Script

| Feature | Fish Script | Rust Implementation |
|---------|-------------|---------------------|
| Rate limiting | ✅ | ✅ |
| Parallel deletion | `xargs -P 15` | Rayon thread pool (15 threads) |
| Error handling | Status codes + stderr | `Result<T, E>` + `anyhow` |
| JSON parsing | `jq` | `serde_json` |
| Colors | `colorme` | `colored` crate |
| Performance | ~Same | ~Same (both spawn `gh` processes) |

## Dependencies

- **serde/serde_json** - JSON parsing
- **colored** - Terminal colors
- **rayon** - Parallel iteration
- **anyhow** - Error handling
- **chrono** - Time calculations

## Error Handling

The script handles:
- Network failures (30s retry)
- API errors (5s retry)
- Rate limit exhaustion (sleep until reset + 10s)
- Secondary rate limits (60s backoff)

## License

MIT
