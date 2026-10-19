// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Azusa CLI — repository drift intelligence.
//!
//! Measures, classifies, explains and plans reconciliation of drift between
//! repository states. The bare `azusa` invocation is the fast path and is
//! equivalent to `azusa drift`.
//!
//! Exit codes: 0 no significant drift · 1 medium · 2 high/critical ·
//! 3 policy violation · 4 analysis failure · 5 invalid invocation.

mod commands;
mod output;

use clap::Parser;
use clap::Subcommand;

use azusa_core::AzusaConfig;

/// Repository drift intelligence: measure, explain and classify how two
/// repository states have diverged.
#[derive(Debug, Parser)]
#[command(name = "azusa", version, about, arg_required_else_help = false)]
struct Cli {
    #[command(flatten)]
    output: output::OutputOpts,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Identity and quick health of a state (default: the worktree).
    Status {
        /// State selector (worktree, head, upstream, a revision, or a path).
        spec: Option<String>,
    },

    /// Dump a full repository state (all facets).
    Inspect {
        /// State selector (default: worktree).
        spec: Option<String>,
    },

    /// Compare two states: what is different? (default command)
    Drift {
        /// State selectors: none = worktree vs upstream; one = worktree vs
        /// spec; two = spec vs spec.
        specs: Vec<String>,
    },

    /// The drift index only.
    Score {
        /// State selectors, as for `drift`.
        specs: Vec<String>,
    },

    /// Why does it matter? Clusters + full evidence for every observation.
    Explain {
        /// State selectors, as for `drift`.
        specs: Vec<String>,
    },

    /// CI gate: drift analysis plus policy enforcement (exit 3 on violation).
    Policy {
        /// The policy subcommand.
        #[command(subcommand)]
        action: PolicyAction,
    },

    /// Write a canonical drift-report artefact under `.azusa/exports/`.
    Export {
        /// State selectors, as for `drift`.
        specs: Vec<String>,
    },

    /// Generate a reconciliation plan (steps + safety gates) and export it
    /// under `.azusa/exports/`. Azusa plans; Kaptaind executes.
    Plan {
        /// State selectors, as for `drift`.
        specs: Vec<String>,
    },

    /// Continuous drift watch: quick stat-only checks with staged
    /// escalation to full analysis (and a reconciliation plan at stage 2).
    Watch {
        /// Seconds between quick checks.
        #[arg(long, default_value_t = 5)]
        interval: u64,

        /// Stop after this many cycles (0 = run until interrupted).
        #[arg(long, default_value_t = 0)]
        max_cycles: u64,
    },
}

#[derive(Debug, Subcommand)]
enum PolicyAction {
    /// Check the working tree (or given states) against the configured
    /// policy.
    Check {
        /// State selectors, as for `drift`.
        specs: Vec<String>,
    },
}

/// Errors that map onto exit codes.
#[derive(Debug)]
enum RunError {
    /// Analysis failure (exit 4).
    Analysis(azusa_core::CoreError),
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Analysis(e) => write!(f, "{e}"),
        }
    }
}

impl From<azusa_core::CoreError> for RunError {
    fn from(value: azusa_core::CoreError) -> Self {
        Self::Analysis(value)
    }
}

fn main() -> std::process::ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            // Distinguish usage errors (5) from --help/----version (0),
            // which clap signals through exit code 0.
            let code = if error.use_stderr() { 5 } else { 0 };
            print!("{error}");
            return std::process::ExitCode::from(code.clamp(0, 5) as u8);
        }
    };

    let root = std::env::current_dir().unwrap_or_else(|_| ".".into());
    let config = match AzusaConfig::load(&root) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("azusa: {error}");
            return std::process::ExitCode::from(4);
        }
    };

    let code = match cli.command.unwrap_or(Command::Drift { specs: Vec::new() }) {
        Command::Status { spec } => commands::status(spec.as_deref(), &config, &cli.output),
        Command::Inspect { spec } => commands::inspect(spec.as_deref(), &cli.output),
        Command::Drift { specs } => commands::drift(&specs, &config, &cli.output),
        Command::Score { specs } => commands::score(&specs, &config, &cli.output),
        Command::Explain { specs } => commands::explain(&specs, &config, &cli.output),
        Command::Policy { action } => match action {
            PolicyAction::Check { specs } => commands::policy(&specs, &config, &cli.output),
        },
        Command::Export { specs } => commands::export(&specs, &config, &cli.output),
        Command::Plan { specs } => commands::plan(&specs, &config, &cli.output),
        Command::Watch {
            interval,
            max_cycles,
        } => commands::watch(interval, max_cycles, &config, &cli.output),
    };

    match code {
        Ok(code) => std::process::ExitCode::from(code.clamp(0, 5) as u8),
        Err(error) => {
            if !cli.output.quiet {
                eprintln!("azusa: {error}");
            }
            std::process::ExitCode::from(4)
        }
    }
}
