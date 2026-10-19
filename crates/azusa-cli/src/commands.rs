// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

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
    let report = compare(&a_ref.spec, &a, &b_ref.spec, &b, config, &notes);
    record_report(&report, resolver.root());
    Ok(report)
}

/// Record a report in the local evidence store and emit ELCI events.
/// Best-effort: analysis never fails because the store is unwritable.
fn record_report(report: &DriftReport, root: &std::path::Path) {
    let store = azusa_store::Store::open(root);
    let timestamp = azusa_elci::utc_now_rfc3339();
    store.append_history(report, &timestamp).ok();
    azusa_elci::EventEmitter::to_file(store.events_path()).emit_for_report(report);
}

/// `azusa watch` — continuous drift monitoring with staged escalation.
///
/// Stage 0 (idle): cheap stat-only quick checks; no analysis. Stage 1: the
/// quick digest changed since the last full analysis — run the full
/// pipeline (history + events). Stage 2 (escalated): the drift grew or is
/// high/critical — additionally generate a reconciliation plan. State
/// persists under `.azusa/watch.json` so restarts resume. Analysis per
/// cycle stays deterministic; only the loop itself is temporal.
pub fn watch(
    interval_secs: u64,
    max_cycles: u64,
    config: &AzusaConfig,
    opts: &OutputOpts,
) -> Result<i32, RunError> {
    let root = std::env::current_dir().unwrap_or_else(|_| ".".into());
    let resolver = StateResolver::new(root.clone());
    let store = azusa_store::Store::open(&root);
    let mut state = store.load_watch_state();
    let mut last_code = 0;

    let cycle_count = |cycle: u64| max_cycles == 0 || cycle <= max_cycles;
    let mut cycle: u64 = 0;
    while cycle_count(cycle + 1) {
        cycle += 1;
        let quick = azusa_fs::FsProvider::new(&root)
            .quick_digest()
            .map_err(|e| RunError::Analysis(azusa_core::CoreError::Provider(format!("watch: {e}"))))?;

        if quick == state.last_full_quick_digest {
            // Stage 0: idle — no analysis, no history churn.
            state.stage = 0;
            state.streak = 0;
            store.save_watch_state(&state);
            if !opts.quiet {
                if opts.machine() {
                    println!(
                        "{}",
                        serde_json::json!({"cycle": cycle, "changed": false, "stage": 0})
                    );
                } else {
                    println!("watch cycle {cycle}: no change (stage 0, idle)");
                }
            }
        } else {
            let report = build_report(&resolver, config, &[])?;
            let grew = state
                .last_score
                .is_some_and(|previous| report.score.overall > previous);
            let high = report.risk.level >= azusa_model::RiskLevel::High;
            let escalated = grew || high;
            let new_stage = if escalated { 2 } else { 1 };
            let previous_stage = state.stage;
            state.stage = new_stage;
            state.streak = if escalated { state.streak + 1 } else { 0 };
            state.last_full_quick_digest = quick;
            state.last_score = Some(report.score.overall);
            report
                .risk
                .level
                .as_str()
                .clone_into(&mut state.last_risk);
            store.save_watch_state(&state);

            let mut plan_path = None;
            if new_stage == 2 && previous_stage < 2 {
                let policy_result = azusa_policy::check(&report, &config.policy);
                let plan = azusa_core::plan::generate(&report, policy_result.is_compliant());
                let timestamp = azusa_elci::utc_now_rfc3339();
                plan_path = azusa_elci::export_plan(&plan, &store.exports_dir(), &timestamp).ok();
                azusa_elci::EventEmitter::to_file(store.events_path()).emit(
                    &azusa_elci::Event::from_report(
                        azusa_elci::events::PLAN_GENERATED,
                        &report,
                        &timestamp,
                    ),
                );
            }

            last_code = output::exit_code(&report, &config.exit);
            if !opts.quiet {
                if opts.machine() {
                    println!(
                        "{}",
                        serde_json::json!({
                            "cycle": cycle,
                            "changed": true,
                            "stage": new_stage,
                            "score": report.score.overall,
                            "risk": report.risk.level.as_str(),
                            "plan": plan_path.as_ref().map(|p| p.display().to_string()),
                        })
                    );
                } else {
                    let stage_note = match new_stage {
                        2 => "escalated — plan generated",
                        _ => "analysed",
                    };
                    println!(
                        "watch cycle {cycle}: change detected (stage {new_stage}, {stage_note})"
                    );
                    println!(
                        "  drift {:.1}/10  risk {}",
                        report.score.overall,
                        report.risk.level.as_str()
                    );
                    if let Some(path) = &plan_path {
                        println!("  plan: {}", path.display());
                    }
                }
            }
        }

        if cycle_count(cycle + 1) && interval_secs > 0 {
            std::thread::sleep(std::time::Duration::from_secs(interval_secs));
        }
    }
    Ok(last_code)
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

/// `azusa policy check` — CI gate: drift plus policy, exit 3 on violation.
pub fn policy(specs: &[String], config: &AzusaConfig, opts: &OutputOpts) -> Result<i32, RunError> {
    let resolver = StateResolver::new(std::env::current_dir().unwrap_or_else(|_| ".".into()));
    let report = build_report(&resolver, config, specs)?;
    let result = azusa_policy::check(&report, &config.policy);

    if !result.is_compliant() {
        let store = azusa_store::Store::open(resolver.root());
        let timestamp = azusa_elci::utc_now_rfc3339();
        let emitter = azusa_elci::EventEmitter::to_file(store.events_path());
        emitter.emit(&azusa_elci::Event::from_report(
            azusa_elci::events::POLICY_FAILED,
            &report,
            &timestamp,
        ));
    }

    let code = if result.is_compliant() {
        output::exit_code(&report, &config.exit)
    } else {
        3
    };
    if opts.quiet {
        return Ok(code);
    }
    if opts.machine() {
        let payload = serde_json::json!({
            "compliant": result.is_compliant(),
            "violations": result.violations,
            "report": report,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
    } else {
        println!("POLICY CHECK — {}", report.repository.name);
        println!("{}", "─".repeat(60));
        println!(
            "drift: {:.1}/10  risk: {}",
            report.score.overall,
            report.risk.level.as_str()
        );
        if result.is_compliant() {
            println!("violations: none — compliant");
        } else {
            println!("violations: {} — NON-COMPLIANT", result.violations.len());
            for violation in &result.violations {
                println!(
                    "  {} [{}] {}: {}",
                    violation.id,
                    violation.severity.as_str(),
                    violation.rule,
                    violation.description
                );
                for reference in violation.references.iter().take(4) {
                    println!("        {reference}");
                }
            }
        }
    }
    Ok(code)
}

/// `azusa export` — canonical drift-report artefact + ELCI events.
pub fn export(specs: &[String], config: &AzusaConfig, opts: &OutputOpts) -> Result<i32, RunError> {
    let resolver = StateResolver::new(std::env::current_dir().unwrap_or_else(|_| ".".into()));
    let report = build_report(&resolver, config, specs)?;
    let store = azusa_store::Store::open(resolver.root());
    let timestamp = azusa_elci::utc_now_rfc3339();
    let path = azusa_elci::export_report(&report, &store.exports_dir(), &timestamp)
        .map_err(|e| RunError::Analysis(azusa_core::CoreError::Provider(format!("export: {e}"))))?;
    let emitter = azusa_elci::EventEmitter::to_file(store.events_path());
    emitter.emit(&azusa_elci::Event::from_report(
        azusa_elci::events::INSPECT_COMPLETED,
        &report,
        &timestamp,
    ));
    if !opts.quiet {
        if opts.machine() {
            println!(
                "{}",
                serde_json::json!({"export": path.display().to_string(), "events": store.events_path().display().to_string()})
            );
        } else {
            println!("exported: {}", path.display());
            println!("events:   {}", store.events_path().display());
        }
    }
    Ok(output::exit_code(&report, &config.exit))
}

/// `azusa plan` — deterministic reconciliation plan (steps + safety
/// gates), exported as a canonical artefact. Azusa plans; Kaptaind
/// executes.
pub fn plan(specs: &[String], config: &AzusaConfig, opts: &OutputOpts) -> Result<i32, RunError> {
    let resolver = StateResolver::new(std::env::current_dir().unwrap_or_else(|_| ".".into()));
    let report = build_report(&resolver, config, specs)?;
    let policy_result = azusa_policy::check(&report, &config.policy);
    let plan = azusa_core::plan::generate(&report, policy_result.is_compliant());

    let store = azusa_store::Store::open(resolver.root());
    let timestamp = azusa_elci::utc_now_rfc3339();
    let path = azusa_elci::export_plan(&plan, &store.exports_dir(), &timestamp)
        .map_err(|e| RunError::Analysis(azusa_core::CoreError::Provider(format!("plan: {e}"))))?;
    let emitter = azusa_elci::EventEmitter::to_file(store.events_path());
    emitter.emit(&azusa_elci::Event::from_report(
        azusa_elci::events::PLAN_GENERATED,
        &report,
        &timestamp,
    ));

    if !opts.quiet {
        if opts.machine() {
            let payload = serde_json::json!({
                "plan": plan,
                "export": path.display().to_string(),
                "events": store.events_path().display().to_string(),
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).unwrap_or_default()
            );
        } else {
            println!(
                "RECONCILIATION PLAN — {} ({} → {})",
                plan.repository, plan.state_a, plan.state_b
            );
            println!("{}", "─".repeat(60));
            println!("{}", plan.summary);
            if !plan.is_executable() {
                println!("BLOCKED — gates must pass before execution");
            }
            for gate in &plan.gates {
                println!(
                    "  {} [{}] {}",
                    gate.id,
                    if gate.blocking { "blocking" } else { "advisory" },
                    gate.condition
                );
                println!("        {}", gate.reason);
            }
            for step in &plan.steps {
                println!(
                    "  {} [{}] {}: {}",
                    step.id,
                    step.kind.as_str(),
                    step.risk.as_str(),
                    step.description
                );
                println!(
                    "        rationale: {}",
                    step.rationale
                );
                if step.requires_approval {
                    println!("        requires human approval (executed by {})", step.executed_by);
                }
            }
            println!("artefact:  {}", path.display());
            println!("events:    {}", store.events_path().display());
        }
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
