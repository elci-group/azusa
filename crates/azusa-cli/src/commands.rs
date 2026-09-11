//! Command implementations.
//!
//! Commands resolve states through [`StateResolver`], run the comparison
//! engine, and render through [`crate::output`]. Exit codes: 0 no
//! significant drift, 1 medium, 2 high/critical (thresholds configurable),
//! 4 analysis failure.
use azusa_core::{AzusaConfig, StateResolver, compare};
use azusa_model::{DriftReport, RepositoryState, StateRef};

use crate::RunError;
use crate::output::{self, OutputOpts};

/// Resolve the pair for a drift-style command from 0–2 positional specs.
fn resolve_pair(
    resolver: &StateResolver,
    specs: &[String],
) -> Result<
    (
        RepositoryState,
        StateRef,
        RepositoryState,
        StateRef,
        Vec<String>,
    ),
    RunError,
> {
    match specs.len() {
        0 => resolver.resolve_default_pair().map_err(RunError::from),
        1 => {
            let (a, a_ref) = resolver.resolve_ref("worktree")?;
            let (b, b_ref) = resolver.resolve_ref(&specs[0])?;
            Ok((a, a_ref, b, b_ref, Vec::new()))
        }
        _ => {
            let (a, a_ref) = resolver.resolve_ref(&specs[0])?;
            let (b, b_ref) = resolver.resolve_ref(&specs[1])?;
            Ok((a, a_ref, b, b_ref, Vec::new()))
        }
    }
}

fn build_report(
    resolver: &StateResolver,
    config: &AzusaConfig,
    specs: &[String],
) -> Result<DriftReport, RunError> {
    let (a, a_ref, b, b_ref, notes) = resolve_pair(resolver, specs)?;
    Ok(compare(&a_ref.spec, &a, &b_ref.spec, &b, config, &notes))
}

/// `azusa drift` — what is different?
pub fn drift(specs: &[String], config: &AzusaConfig, opts: &OutputOpts) -> Result<i32, RunError> {
    let resolver = StateResolver::new(std::env::current_dir().unwrap_or_else(|_| ".".into()));
    let report = build_report(&resolver, config, specs)?;
    if !opts.quiet {
        println!("{}", output::render_report(&report, opts));
    }
    Ok(output::exit_code(&report, &config.exit))
}

/// `azusa explain` — why does it matter? Full evidence, unclipped.
pub fn explain(specs: &[String], config: &AzusaConfig, opts: &OutputOpts) -> Result<i32, RunError> {
    let resolver = StateResolver::new(std::env::current_dir().unwrap_or_else(|_| ".".into()));
    let report = build_report(&resolver, config, specs)?;
    if !opts.quiet {
        if opts.machine() {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            );
        } else {
            println!("{}", output::report_to_human(&report));
        }
    }
    Ok(output::exit_code(&report, &config.exit))
}

/// `azusa score` — the drift index only.
pub fn score(specs: &[String], config: &AzusaConfig, opts: &OutputOpts) -> Result<i32, RunError> {
    let resolver = StateResolver::new(std::env::current_dir().unwrap_or_else(|_| ".".into()));
    let report = build_report(&resolver, config, specs)?;
    if opts.quiet {
        return Ok(output::exit_code(&report, &config.exit));
    }
    if opts.machine() {
        println!(
            "{}",
            serde_json::to_string(&report.score).unwrap_or_default()
        );
    } else {
        println!("DRIFT INDEX");
        println!("───────────");
        println!("Overall     {:.1} / 10", report.score.overall);
        for layer in azusa_model::Layer::all() {
            if let Some(value) = report.score.layers.get(&layer) {
                println!("{:<11} {:>5.2}", layer.as_str(), value);
            }
        }
    }
    Ok(output::exit_code(&report, &config.exit))
}

/// `azusa status` — identity and quick health of the working state.
pub fn status(
    spec: Option<&str>,
    config: &AzusaConfig,
    opts: &OutputOpts,
) -> Result<i32, RunError> {
    let _ = config;
    let resolver = StateResolver::new(std::env::current_dir().unwrap_or_else(|_| ".".into()));
    let spec = spec.unwrap_or("worktree");
    let state = resolver.resolve(spec)?;
    if !opts.quiet {
        println!("{}", output::render_state(&state, opts));
    }
    Ok(0)
}

/// `azusa inspect` — full state dump (all facets).
pub fn inspect(spec: Option<&str>, opts: &OutputOpts) -> Result<i32, RunError> {
    let resolver = StateResolver::new(std::env::current_dir().unwrap_or_else(|_| ".".into()));
    let spec = spec.unwrap_or("worktree");
    let state = resolver.resolve(spec)?;
    if !opts.quiet {
        if opts.machine() {
            println!(
                "{}",
                serde_json::to_string_pretty(&state).unwrap_or_default()
            );
        } else {
            println!("{}", output::render_state(&state, opts));
            println!(
                "\nstructure:    {} module(s)",
                state.structure.modules.len()
            );
            println!(
                "dependencies: {} declared",
                state.dependencies.entries.len()
            );
            println!("lockfiles:    {}", state.dependencies.lockfiles.len());
            println!("config files: {}", state.configuration.files.len());
        }
    }
    Ok(0)
}
