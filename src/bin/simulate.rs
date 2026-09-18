// simulate.rs — Metatron Dynamics, Inc.
// abr-primary-relational-cell V0.1
// Convention build — WeAreDevelopers San Jose
//
// Entry point. Runs all ten fixtures (F1–F10), produces:
//   1. Human-readable comparison table to stdout
//   2. Machine-readable JSON run record to results/
//
// Kernel authority: operators.rs V7, derived_invariants.rs V4.1
// Build plan authority: V1.2 G1–G13
//
// B absent. ABR operators absent. No persistence. No physical claims.

use std::fs;
use chrono::Local;
use serde_json;

use prc::topology::*;
use prc::node_field::{NodeField, ObservationClass};
use prc::admissibility::{run_experiment, ExperimentRecord};
use prc::experiment::{
    ProcessDirection, run_single_step,
    run_intervention_comparison, run_closure_removal,
};
use prc::kernel::RhoP;

fn si() -> ObservationClass { ObservationClass::SimulatedInput }

fn rho_p_label(rho: &Option<RhoP>) -> String {
    match rho {
        None => "—".to_string(),
        Some(RhoP::Determined { rank_sigma, cx }) =>
            format!("Determined (rank={}, C_X={})", rank_sigma, cx),
        Some(RhoP::CeilingBounded { rank_sigma, cx }) =>
            format!("CeilingBounded (rank={}, C_X={})", rank_sigma, cx),
        Some(RhoP::Undefined { .. }) => "Undefined".to_string(),
    }
}

fn admissibility_label(rec: &ExperimentRecord) -> String {
    if let Some(fm) = &rec.failure_mode {
        return format!("FAILURE: {:?}", fm);
    }
    if let Some(af) = &rec.admissibility_finding {
        return format!("Finding: {:?}", af);
    }
    "Admissible".to_string()
}

fn print_separator() {
    println!("{}", "─".repeat(90));
}

fn print_record(rec: &ExperimentRecord) {
    println!(
        "  {:30} | C_X={} | k={} | rank_Δ={} | rank_Σ={}",
        rec.fixture_id,
        rec.cx,
        rec.k,
        rec.rank_delta.map(|r| r.to_string()).unwrap_or("—".to_string()),
        rec.rank_sigma.map(|r| r.to_string()).unwrap_or("—".to_string()),
    );
    println!(
        "  {:30}   ρ_P: {}",
        "",
        rho_p_label(&rec.rho_p)
    );
    println!(
        "  {:30}   expr_cond: {}  |  cycle: {}  |  {}",
        "",
        rec.expression_condition.map(|b| b.to_string()).unwrap_or("—".to_string()),
        rec.cycle_detected,
        admissibility_label(rec),
    );
}

fn main() {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║  ABR Primary Relational Cell Simulator — V0.1                                      ║");
    println!("║  Metatron Dynamics, Inc. — WeAreDevelopers San Jose                                ║");
    println!("║  Kernel: operators.rs V7 | derived_invariants.rs V4.1 | Build plan V1.2            ║");
    println!("║  B absent. Physical fabrication not claimed.                                        ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════════════╝");
    println!();

    let mut all_records: Vec<serde_json::Value> = vec![];

    // ── SECTION 1: Baseline topology matrix ──────────────────────────────

    println!("SECTION 1 — Baseline topology matrix (F1–F7, k=1)");
    print_separator();

    let baselines: Vec<(&str, DeclaredRelations, Vec<f64>)> = vec![
        ("F1_PRC2_pair",       fixture_f1_pair(),               vec![1.0, 2.0]),
        ("F2_chain3",          fixture_f2_chain3(),             vec![1.0, 2.0, 3.0]),
        ("F3_3LCT",            fixture_f3_closed_relational_3(), vec![1.0, 2.0, 3.0]),
        ("F4_chain4",          fixture_f4_chain4(),             vec![1.0, 2.0, 3.0, 4.0]),
        ("F5_branch",          fixture_f5_branch(),             vec![1.0, 2.0, 3.0, 4.0]),
        ("F6_convergent",      fixture_f6_convergent(),         vec![1.0, 2.0, 3.0, 4.0]),
        ("F7_diamond",         fixture_f7_diamond(),            vec![1.0, 2.0, 3.0, 4.0]),
    ];

    for (id, rel, vals) in &baselines {
        let x = NodeField::single(vals.clone(), si());
        let rec = run_single_step(id, &x, rel, &ProcessDirection::SingleStep)
            .expect("single step must succeed");
        print_record(&rec);
        print_separator();
        all_records.push(serde_json::to_value(&rec).unwrap());
    }

    // ── SECTION 2: Three-locus closed relational topology — k scaling ─────

    println!();
    println!("SECTION 2 — Three-locus closed relational topology (F3) — k=1,2,3");
    println!("  Terminology: '3LCT' = three-locus closed relational topology");
    println!("               V={{A,B,C}}, R={{A→B, B→C, C→A}}");
    println!("  C_X=3: ρ_P Determined requires k≥3");
    print_separator();

    for k in 1usize..=3 {
        let rel = fixture_f3_closed_relational_3();
        // k components: use gradient offset per component
        let field: Vec<Vec<f64>> = (0..k)
            .map(|c| vec![1.0 + c as f64, 2.0 + c as f64, 3.0 + c as f64])
            .collect();
        let x = NodeField::new(field, si());
        let fixture_id = format!("F3_3LCT_k{}", k);
        let rec = run_experiment(&fixture_id, &x, &rel);
        print_record(&rec);
        print_separator();
        all_records.push(serde_json::to_value(&rec).unwrap());
    }

    // ── SECTION 3: Matched intervention comparison (F8 vs F9) ────────────

    println!();
    println!("SECTION 3 — Matched intervention comparison");
    println!("  F8: disturbance at locus A, 3LCT topology");
    println!("  F9: disturbance at locus A, chain topology (matched control)");
    println!("  Disturbance: +0.5 at node 0 (locus A)");
    println!("  Outputs recorded — no predetermined result (G11)");
    print_separator();

    let rel_f8 = fixture_f8_closed_relational_3_intervention();
    let rel_f9 = fixture_f9_chain3_intervention();
    let comparison = run_intervention_comparison(
        &rel_f8, &rel_f9,
        vec![1.0, 2.0, 3.0],
        0,   // disturbance at node 0 (locus A)
        0.5, // disturbance magnitude
    );

    println!("  F8 — 3LCT arm:");
    print_record(&comparison.primary_arm);
    if let Some(sigma) = &comparison.primary_arm.sigma_values {
        println!("        Σ values: {:?}", sigma);
    }
    print_separator();
    println!("  F9 — Chain control arm:");
    print_record(&comparison.control_arm);
    if let Some(sigma) = &comparison.control_arm.sigma_values {
        println!("        Σ values: {:?}", sigma);
    }
    print_separator();
    println!("  Note: {}", comparison.comparison_note);
    print_separator();

    all_records.push(serde_json::to_value(&comparison.primary_arm).unwrap());
    all_records.push(serde_json::to_value(&comparison.control_arm).unwrap());

    // ── SECTION 4: Topological change — F10 closure removal ──────────────

    println!();
    println!("SECTION 4 — Topological change: closure removal (F10)");
    println!("  F3:  3LCT with C→A declared (C_X=3)");
    println!("  F10: C→A removed — open chain A→B→C (C_X=1)");
    println!("  This is a binary declaration change — not scalar weakening (G12)");
    print_separator();

    let rel_closed = fixture_f3_closed_relational_3();
    let rel_open   = fixture_f10_closure_removed();
    let (rec_closed, rec_open) = run_closure_removal(
        &rel_closed, &rel_open,
        vec![1.0, 2.0, 3.0],
    );

    println!("  F3 — 3LCT (C→A present):");
    print_record(&rec_closed);
    print_separator();
    println!("  F10 — C→A removed:");
    print_record(&rec_open);
    print_separator();

    all_records.push(serde_json::to_value(&rec_closed).unwrap());
    all_records.push(serde_json::to_value(&rec_open).unwrap());

    // ── SECTION 5: Open conditions ────────────────────────────────────────

    println!();
    println!("SECTION 5 — Open conditions (not closed by this build)");
    print_separator();
    println!("  OC-PRC-1: What observable conditions distinguish a relational");
    println!("            information cell from an arbitrary declared relational");
    println!("            structure? Not closed until V0.6 at earliest.");
    println!();
    println!("  OC-PRC-2: Minimum topology for expression_condition = true");
    println!("            consistently across declared observation range.");
    println!();
    println!("  OC-PRC-3: Does intervention at non-adjacent locus produce change");
    println!("            in Σ at non-directly-intervened edge?");
    println!();
    println!("  OC-ρP-1, OC-ρP-2: Carried from kernel unchanged.");
    println!();
    println!("  Physical correspondence: Declared open. No material implementation");
    println!("  selected. Quantum dots are a candidate only.");
    println!();
    println!("  Terminology discipline: 'Three-locus closed relational topology'");
    println!("  (3LCT) = abstract directed relational structure.");
    println!("  Physical geometry is a separate level requiring measurement");
    println!("  correspondence M to establish directed edge provenance.");
    print_separator();

    // ── Write JSON run record ─────────────────────────────────────────────

    let run_record = serde_json::json!({
        "simulator": "abr-primary-relational-cell",
        "version": "V0.1",
        "build_plan": "V1.2",
        "kernel_authority": "operators.rs V7 / derived_invariants.rs V4.1",
        "timestamp": timestamp,
        "convention": "WeAreDevelopers San Jose",
        "b_absent": true,
        "abr_operators_absent": true,
        "physical_fabrication_claimed": false,
        "records": all_records,
    });

    fs::create_dir_all("results").expect("could not create results/");
    let outpath = format!("results/prc_sim_v01_{}.json", timestamp);
    fs::write(&outpath, serde_json::to_string_pretty(&run_record).unwrap())
        .expect("could not write run record");

    println!();
    println!("Run record written: {}", outpath);
    println!();
    println!("── Convention claim boundary ──────────────────────────────────────");
    println!("   This simulator calculates from declared observables and relations.");
    println!("   The first physical experiment — comparing a three-locus closed");
    println!("   relational topology with an open-chain control inside a managed");
    println!("   thermal and electromagnetic boundary — is the next declared stage.");
    println!("   Physical results are not claimed.");
    println!();
}
