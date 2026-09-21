// v02_sweep.rs — Metatron Dynamics, Inc.
// abr-primary-relational-cell V0.2.0
//
// Governing question:
//   What relational expression is observed across the declared k∈{1,2} matrix
//   and systematic intervention atlas?
//
// Runs three sweep categories:
//   1. k-matrix: k ∈ {1,2} × rho_base ∈ {0.1,0.3,0.5} across all F1–F7 fixtures
//   2. Intervention atlas: F8 (3LCT) vs F9 (chain), full disturbance space
//   3. Closure removal: F3 (closed) vs F10 (open) across k ∈ {1,2} × rho_base sweep
//
// Output: single JSON object written to results/v02_sweep_results.json
//
// Input fields: Origin-declared 2026-09-20 (ascending + cyclic rotation).
// No rank guarantee asserted. Kernel observes what results.

use prc::kernel::RHO_BASE_SWEEP;
use prc::topology::{
    fixture_f1_pair, fixture_f2_chain3, fixture_f3_closed_relational_3,
    fixture_f4_chain4, fixture_f5_branch, fixture_f6_convergent, fixture_f7_diamond,
    fixture_f8_closed_relational_3_intervention, fixture_f9_chain3_intervention,
    fixture_f10_closure_removed,
};
use prc::experiment::{
    sweep_k_matrix, sweep_intervention_atlas, sweep_closure_removal,
};
use chrono::Utc;
use serde_json::json;

fn main() {
    eprintln!("abr-primary-relational-cell V0.2.0 — sweep run");
    eprintln!("Governing question: What relational expression is observed");
    eprintln!("  across the declared k∈{{1,2}} matrix and systematic intervention atlas?");
    eprintln!("rho_base sweep: {:?}", RHO_BASE_SWEEP);
    eprintln!();

    // ── 1. k-matrix sweep: F1–F7 ─────────────────────────────────────────────

    let fixtures: Vec<(&str, prc::topology::DeclaredRelations)> = vec![
        ("F1_Pair",       fixture_f1_pair()),
        ("F2_Chain3",     fixture_f2_chain3()),
        ("F3_3LCT",       fixture_f3_closed_relational_3()),
        ("F4_Chain4",     fixture_f4_chain4()),
        ("F5_Branch",     fixture_f5_branch()),
        ("F6_Convergent", fixture_f6_convergent()),
        ("F7_Diamond",    fixture_f7_diamond()),
    ];

    let mut k_matrix_results = Vec::new();
    for (label, rel) in &fixtures {
        eprintln!("k-matrix sweep: {}", label);
        let records = sweep_k_matrix(label, rel, &RHO_BASE_SWEEP);
        k_matrix_results.push(json!({
            "fixture_label": label,
            "records": records,
        }));
    }

    // ── 2. Intervention atlas: F8 (3LCT) vs F9 (Chain3) ─────────────────────

    eprintln!("Intervention atlas: F8 (3LCT) vs F9 (Chain3)");
    let disturbance_nodes = vec![0usize, 1, 2];
    let disturbance_deltas = vec![0.1f64, 0.5, 1.0, 2.0];
    let base_values = vec![1.0f64, 2.0, 3.0];  // declared k=1 field for n=3

    let comparisons = sweep_intervention_atlas(
        &fixture_f8_closed_relational_3_intervention(),
        &fixture_f9_chain3_intervention(),
        base_values,
        &disturbance_nodes,
        &disturbance_deltas,
        &RHO_BASE_SWEEP,
    );

    // ── 3. Closure removal: F3 (closed 3LCT) vs F10 (C→A removed) ───────────

    eprintln!("Closure removal sweep: F3 (3LCT closed) vs F10 (C→A removed)");
    let closure_pairs = sweep_closure_removal(
        &fixture_f3_closed_relational_3(),
        &fixture_f10_closure_removed(),
        &RHO_BASE_SWEEP,
    );

    let closure_records: Vec<serde_json::Value> = closure_pairs.iter()
        .map(|(closed, open)| json!({
            "closed": closed,
            "open":   open,
        }))
        .collect();

    // ── Assemble single output object ─────────────────────────────────────────

    let total_k_records = fixtures.len() * 2 * RHO_BASE_SWEEP.len();
    let total_comparisons = disturbance_nodes.len() * disturbance_deltas.len() * RHO_BASE_SWEEP.len();
    let total_closure_pairs = 2 * RHO_BASE_SWEEP.len();

    let output = json!({
        "run_metadata": {
            "version": "0.2.0",
            "timestamp": Utc::now().to_rfc3339(),
            "governing_question": "What relational expression is observed across the declared k∈{1,2} matrix and systematic intervention atlas?",
            "rho_base_sweep": RHO_BASE_SWEEP,
            "input_fields": {
                "declaration_date": "2026-09-20",
                "k1_rule": "ascending sequence [1..n]",
                "k2_rule": "component_0: ascending sequence; component_1: cyclic rotation",
                "declared_node_counts": [2, 3, 4],
                "note": "Explicitly declared experimental inputs — not rank-engineered"
            },
            "record_counts": {
                "k_matrix_records": total_k_records,
                "intervention_comparisons": total_comparisons,
                "closure_removal_pairs": total_closure_pairs
            }
        },
        "k_matrix": k_matrix_results,
        "intervention_atlas": {
            "description": format!(
                "F8 (3LCT) vs F9 (Chain3) — nodes {:?} × deltas {:?} × rho_base {:?}",
                disturbance_nodes, disturbance_deltas, RHO_BASE_SWEEP
            ),
            "disturbance_nodes": disturbance_nodes,
            "disturbance_deltas": disturbance_deltas,
            "comparisons": comparisons,
        },
        "closure_removal": {
            "description": "F3 (3LCT closed) vs F10 (C→A removed) — binary topological change, not scalar weakening",
            "pairs": closure_records,
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)
        .expect("output serialization must succeed"));

    eprintln!();
    eprintln!("V0.2.0 sweep complete.");
    eprintln!("  k-matrix records:        {}", total_k_records);
    eprintln!("  intervention comparisons: {}", total_comparisons);
    eprintln!("  closure removal pairs:    {}", total_closure_pairs);
}
