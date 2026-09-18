// simulate.rs — Metatron Dynamics, Inc.
// abr-primary-relational-cell V0.1.2
// Convention build — WeAreDevelopers San Jose
//
// Runs all ten fixtures across the full declared rho_base range [0.1, 0.5]
// at sweep values 0.1, 0.3, 0.5.
//
// Origin declaration (18 Sep 2026):
//   rho_base operating range for Primary Region simulator: [0.1, 0.5]
//   Convention sweep: [0.1, 0.3, 0.5]

use std::fs;
use chrono::Local;
use prc::topology::*;
use prc::node_field::{NodeField, ObservationClass};
use prc::admissibility::{run_experiment, ExperimentRecord};
use prc::experiment::{
    ProcessDirection, run_single_step,
    run_intervention_comparison, run_closure_removal,
};
use prc::kernel::{RhoP, FailureMode, RHO_BASE_SWEEP, RHO_BASE_MIN, RHO_BASE_MAX};

fn si() -> ObservationClass { ObservationClass::SimulatedInput }

fn rho_p_label(rho: &Option<RhoP>) -> String {
    match rho {
        None => "—".to_string(),
        Some(RhoP::Determined { value, n_components, propagation_capacity }) =>
            format!("Determined  ρ_P={:.4}  k={}  C_X={}", value, n_components, propagation_capacity),
        Some(RhoP::CeilingBounded { value, ceiling, n_components, propagation_capacity }) =>
            format!("CeilingBounded  ρ_P={:.4}  ceil={:.4}  k={}  C_X={}", value, ceiling, n_components, propagation_capacity),
        Some(RhoP::Undefined { reason }) =>
            format!("Undefined ({:?})", reason),
    }
}

fn status_label(rec: &ExperimentRecord) -> &'static str {
    match &rec.failure_mode {
        Some(FailureMode::DifferentiationCollapse) => "FM1 DifferentiationCollapse",
        Some(FailureMode::RelationalIsolation)     => "FM2 RelationalIsolation",
        None => match &rec.admissibility_finding {
            Some(_) => "Finding: CirculationCancellation",
            None    => "Admissible",
        },
    }
}

fn sep() { println!("{}", "─".repeat(96)); }

fn print_record(rec: &ExperimentRecord) {
    println!("  {:38} ρ_base={:.1} | C_X={} | k={} | rank_Δ={} | rank_Σ={}",
        rec.fixture_id, rec.rho_base, rec.cx, rec.k,
        rec.rank_delta.map(|r| r.to_string()).unwrap_or("—".into()),
        rec.rank_sigma.map(|r| r.to_string()).unwrap_or("—".into()),
    );
    println!("  {:38} ρ_P: {}", "", rho_p_label(&rec.rho_p));
    println!("  {:38} expr={:5}  antisym={:5}  cycle={}  → {}",
        "",
        rec.expression_condition.map(|b| b.to_string()).unwrap_or("—".into()),
        rec.has_nonzero_antisym.map(|b| b.to_string()).unwrap_or("—".into()),
        rec.cycle_detected,
        status_label(rec),
    );
}

fn main() {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║  ABR Primary Relational Cell Simulator — V0.1.2                                                ║");
    println!("║  Metatron Dynamics, Inc. — WeAreDevelopers San Jose                                            ║");
    println!("║  Kernel: operators.rs V7 | derived_invariants.rs V4.1 | Build plan V1.2                        ║");
    println!("║  B absent. Physical fabrication not claimed.                                                    ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Origin declaration — rho_base:");
    println!("    Admissible Primary Region range: [{}, {}]", RHO_BASE_MIN, RHO_BASE_MAX);
    println!("    Rationale: lower bound preserves nonzero antisymmetric term;");
    println!("               upper bound keeps ρ_P ≪ 1 (Primary Region, below OC-ρP-1 threshold).");
    println!("    Convention sweep: {:?}", RHO_BASE_SWEEP);
    println!();

    let mut all_records: Vec<serde_json::Value> = vec![];

    // ── SECTION 1: Baseline topology matrix across rho_base sweep ─────────
    println!("SECTION 1 — Baseline topology matrix (F1–F7, k=1) × rho_base sweep");
    sep();

    let baselines: Vec<(&str, DeclaredRelations, Vec<f64>)> = vec![
        ("F1_PRC2_pair",    fixture_f1_pair(),                vec![1.0, 2.0]),
        ("F2_chain3",       fixture_f2_chain3(),              vec![1.0, 2.0, 3.0]),
        ("F3_3LCT",         fixture_f3_closed_relational_3(), vec![1.0, 2.0, 3.0]),
        ("F4_chain4",       fixture_f4_chain4(),              vec![1.0, 2.0, 3.0, 4.0]),
        ("F5_branch",       fixture_f5_branch(),              vec![1.0, 2.0, 3.0, 4.0]),
        ("F6_convergent",   fixture_f6_convergent(),          vec![1.0, 2.0, 3.0, 4.0]),
        ("F7_diamond",      fixture_f7_diamond(),             vec![1.0, 2.0, 3.0, 4.0]),
    ];

    for (id, rel, vals) in &baselines {
        for &rb in &RHO_BASE_SWEEP {
            let x = NodeField::single(vals.clone(), si());
            let rec = run_single_step(id, &x, rel, &ProcessDirection::SingleStep, rb)
                .expect("single step must succeed");
            print_record(&rec);
            all_records.push(serde_json::to_value(&rec).unwrap());
        }
        sep();
    }

    // ── SECTION 2: 3LCT k scaling at rho_base=0.3 ────────────────────────
    println!();
    println!("SECTION 2 — 3LCT k=1,2,3 at rho_base=0.3  (V={{A,B,C}}, R={{A→B,B→C,C→A}}, C_X=3)");
    println!("  CeilingBounded while k/C_X < 1; Determined at k/C_X ≥ 1");
    sep();

    let k3_fields: Vec<(usize, Vec<Vec<f64>>)> = vec![
        (1, vec![vec![1.0, 2.0, 3.0]]),
        (2, vec![vec![1.0, 2.0, 3.0], vec![4.0, 1.0, 2.0]]),
        (3, vec![vec![1.0, 2.0, 3.0], vec![4.0, 1.0, 2.0], vec![3.0, 5.0, 1.0]]),
    ];
    for (k, fields) in &k3_fields {
        let rel = fixture_f3_closed_relational_3();
        let x = NodeField::new(fields.clone(), si());
        let rec = run_experiment(&format!("F3_3LCT_k{}", k), &x, &rel, 0.3);
        print_record(&rec);
        all_records.push(serde_json::to_value(&rec).unwrap());
    }
    sep();

    // ── SECTION 3: Matched intervention comparison across sweep ───────────
    println!();
    println!("SECTION 3 — Matched intervention: F8 (3LCT) vs F9 (chain), disturbance +0.5 at A");
    sep();

    for &rb in &RHO_BASE_SWEEP {
        println!("  rho_base={:.1}", rb);
        let cmp = run_intervention_comparison(
            &fixture_f8_closed_relational_3_intervention(),
            &fixture_f9_chain3_intervention(),
            vec![1.0, 2.0, 3.0], 0, 0.5, rb,
        );
        print_record(&cmp.primary_arm);
        if let Some(s) = &cmp.primary_arm.sigma_values {
            println!("        Σ: {:?}", s);
        }
        if let Some(a) = &cmp.primary_arm.antisym_values {
            println!("        antisym: {:?}", a);
        }
        print_record(&cmp.control_arm);
        if let Some(s) = &cmp.control_arm.sigma_values {
            println!("        Σ: {:?}", s);
        }
        if let Some(a) = &cmp.control_arm.antisym_values {
            println!("        antisym: {:?}", a);
        }
        all_records.push(serde_json::to_value(&cmp.primary_arm).unwrap());
        all_records.push(serde_json::to_value(&cmp.control_arm).unwrap());
        sep();
    }

    // ── SECTION 4: Topological change F10 ────────────────────────────────
    println!();
    println!("SECTION 4 — Topological change: C→A declared removed (F10) at rho_base=0.3");
    println!("  Binary declaration change — not scalar weakening (G12)");
    sep();

    let (rc, ro) = run_closure_removal(
        &fixture_f3_closed_relational_3(),
        &fixture_f10_closure_removed(),
        vec![1.0, 2.0, 3.0], 0.3,
    );
    println!("  F3 (C→A present, C_X=3):"); print_record(&rc);
    println!("  F10 (C→A removed, C_X=1):"); print_record(&ro);
    all_records.push(serde_json::to_value(&rc).unwrap());
    all_records.push(serde_json::to_value(&ro).unwrap());
    sep();

    // ── SECTION 5: Open conditions ────────────────────────────────────────
    println!();
    println!("SECTION 5 — Open conditions (not closed by this build)");
    sep();
    for line in [
        "OC-PRC-1: Observable conditions distinguishing relational information cell from arbitrary structure.",
        "OC-PRC-2: Minimum topology for expression_condition=true consistently.",
        "OC-PRC-3: Non-adjacent locus intervention propagation through Σ.",
        "OC-ρP-1:  ABR activation threshold — not yet derived as computable criterion.",
        "OC-ρP-2:  CeilingBounded not admissible as Primary Region finding.",
        "Physical: No material implementation selected. Quantum dots candidate only.",
        "Terminology: '3LCT' = abstract directed relational structure.",
        "             Physical geometry is a separate level requiring M.",
    ] { println!("  {}", line); }
    sep();

    // ── Write JSON run record ─────────────────────────────────────────────
    let run_record = serde_json::json!({
        "simulator": "abr-primary-relational-cell",
        "version": "V0.1.2",
        "build_plan": "V1.2",
        "kernel_authority": "operators.rs V7 / derived_invariants.rs V4.1",
        "timestamp": timestamp,
        "convention": "WeAreDevelopers San Jose",
        "b_absent": true,
        "abr_operators_absent": true,
        "physical_fabrication_claimed": false,
        "rho_base_declaration": {
            "origin": "Robin Macomber / Metatron Dynamics, Inc.",
            "date": "2026-09-18",
            "admissible_range": [RHO_BASE_MIN, RHO_BASE_MAX],
            "convention_sweep": RHO_BASE_SWEEP,
            "rationale": "Lower bound preserves nonzero antisymmetric term; upper bound keeps rho_P << 1 (Primary Region, below OC-rhoP-1 threshold)."
        },
        "records": all_records,
    });

    fs::create_dir_all("results").expect("could not create results/");
    let outpath = format!("results/prc_sim_v012_{}.json", timestamp);
    fs::write(&outpath, serde_json::to_string_pretty(&run_record).unwrap())
        .expect("could not write run record");

    println!();
    println!("Run record written: {}", outpath);
    println!();
    println!("── Convention claim boundary ──────────────────────────────────────────");
    println!("   Simulator calculates from declared observables and relations only.");
    println!("   First physical experiment (3LCT vs open-chain control, managed");
    println!("   thermal/EM boundary) is the next declared stage. No physical claims.");
    println!();
}
