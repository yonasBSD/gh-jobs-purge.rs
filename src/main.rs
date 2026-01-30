use std::{process::Command, thread, time::Duration};

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use gh_jobs_purge::{
    calculate_wait_seconds, check_for_secondary_rate_limit, check_rate_limit,
    fetch_runs_with_statuses, parse_and_validate_statuses, should_hibernate,
};
use rayon::prelude::*;

/// GitHub Actions workflow run purge tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Comma-separated list of statuses to filter runs
    ///
    /// Runtime statuses (active runs):
    ///   queued, in-progress, requested, waiting, pending
    ///
    /// Conclusion statuses (finished runs):
    ///   success, failure, cancelled, skipped, neutral, stale, timed-out,
    /// action-required
    ///
    /// Catch-all:
    ///   completed (all finished runs)
    ///
    /// Note: Use dashes (-) or underscores (_) interchangeably (e.g.,
    /// in-progress or in_progress)
    #[arg(short, long, default_value = "completed", value_name = "STATUS")]
    status: String,
}

/// Delete a single GitHub Action run
fn delete_run(run_id: i64) -> Result<()> {
    let output = Command::new("gh")
        .args(["run", "delete", &run_id.to_string()])
        .output()
        .context("Failed to execute gh run delete")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Delete failed for run {}: {}", run_id, stderr);
    }

    Ok(())
}

/// Delete runs in parallel and check for secondary rate limit errors
fn delete_runs_parallel(run_ids: &[i64]) -> Result<bool> {
    // Use a thread-safe container to collect errors
    let errors: Vec<_> = run_ids
        .par_iter()
        .map(|&id| delete_run(id))
        .filter_map(|result| result.err())
        .collect();

    // Check if any error mentions secondary rate limit
    Ok(check_for_secondary_rate_limit(&errors))
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Parse and validate the status filter
    let statuses = parse_and_validate_statuses(&args.status).context("Invalid status argument")?;

    println!(
        "{}",
        "🚀 GitHub Run Purge - Rust Edition".bright_cyan().bold()
    );
    println!(
        "{} Filtering by status: {}",
        "🎯".cyan(),
        statuses.join(", ").cyan().bold()
    );
    println!();

    loop {
        // --- 1. PRE-FLIGHT QUOTA CHECK 🛡️ ---
        let rate_limit = match check_rate_limit() {
            Ok(rl) => rl,
            Err(e) => {
                println!(
                    "{} Cannot reach GitHub API: {}",
                    "❌".red(),
                    e.to_string().red()
                );
                println!("{} Checking network/lockout...", "⏳".yellow());
                thread::sleep(Duration::from_secs(30));
                continue;
            },
        };

        // If credits are low, enter hibernation mode 😴
        if should_hibernate(rate_limit.remaining, 50) {
            let current_time = chrono::Utc::now().timestamp();
            let wait_seconds = calculate_wait_seconds(rate_limit.reset, current_time);
            let wait_minutes = wait_seconds / 60;

            println!(
                "{} API QUOTA EXHAUSTED ({} left).",
                "🚫".red(),
                rate_limit.remaining.to_string().red().bold()
            );
            println!(
                "{} Hibernating for {} minute(s) until reset...",
                "⏳".yellow(),
                wait_minutes.to_string().yellow().bold()
            );

            thread::sleep(Duration::from_secs((wait_seconds + 10) as u64));
            continue;
        }

        // --- 2. FETCH RUNS 🔍 ---
        println!(
            "{} Quota healthy ({} left). Fetching runs...",
            "⚖️".cyan(),
            rate_limit.remaining.to_string().cyan().bold()
        );

        let run_ids = match fetch_runs_with_statuses(&statuses) {
            Ok(runs) => runs,
            Err(e) => {
                println!(
                    "{} Error fetching runs: {}",
                    "⚠️".red(),
                    e.to_string().red()
                );
                thread::sleep(Duration::from_secs(5));
                continue;
            },
        };

        // Check if we're done
        if run_ids.is_empty() {
            println!(
                "{} Success: No more runs found with status: {}!",
                "✨".green(),
                statuses.join(", ").green().bold()
            );
            break;
        }

        // --- 3. DELETE RUNS 🚀 ---
        println!(
            "{} Deleting {} runs in parallel...",
            "🔨".blue(),
            run_ids.len().to_string().blue().bold()
        );

        // Configure rayon to use max 15 threads for this operation
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(15)
            .build()
            .context("Failed to create thread pool")?;

        let hit_secondary_limit = pool.install(|| delete_runs_parallel(&run_ids))?;

        if hit_secondary_limit {
            println!(
                "{} Secondary rate limit hit (moving too fast!).",
                "🐢".red()
            );
            println!("{} Taking a 60s nap to appease GitHub...", "⏳".yellow());
            thread::sleep(Duration::from_secs(60));
            continue;
        }

        // Short breather to stay under the radar 🌬️
        println!("{} Batch cleared. Polling for more...", "✅".cyan());
        thread::sleep(Duration::from_secs(2));
    }

    Ok(())
}
