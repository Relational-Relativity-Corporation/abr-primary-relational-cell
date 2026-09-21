// experiment.rs — Metatron Dynamics, Inc.
// abr-primary-relational-cell V0.2.0
//
// ProcessDirection gate (G8): MultiStep is structurally blocked.
// Σ(Δ(x_r)) does not imply x_{r+1}. Multi-step evaluation requires
// explicit Origin declaration — returning Err until that structure
// is declared and authorized. V0.1/V0.2 operates on single declared steps only.
//
// V0.2 additions:
//   - Origin-declared input fields for k=1 and k=2 at n=2,3,4 (declared 2026-09-20)
//   - sweep_k_matrix(): k ∈ {1,2} × rho_base sweep across a single topology
//   - sweep_intervention_atlas(): full disturbance_node × disturbance_delta × rho_base sweep
//   - sweep_closure_removal(): F3 vs F10 across k ∈ {1,2} × rho_base sweep
//
// Governing question (V0.2):
//   What relational expression is observed across the declared k∈{1,2} matrix
//   and systematic intervention atlas?
//
// Input field declaration (Origin, 2026-09-20):
//   Fields are explicitly declared experimental inputs — not rank-engineered.
//   k=1: ascending sequence. k=2: ascending sequence + cyclic rotation.
//   Values provide differentiated finite observations; no rank guarantee is
//   asserted or intended.
//
// G14: sweep_k_matrix() produces exactly k_count × rho_base_sweep.len() records.
// G15: k in each record equals x.n_components — never inferred independently.
// G16: intervention atlas records are paired — primary and control differ by
//      topology label.

use serde::{Deserialize, Serialize};
use crate::node_field::{NodeField, ObservationClass};
use crate::topology::DeclaredRelations;
use crate::admissibility::{run_experiment, ExperimentRecord};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProcessDirection {
    /// Single declared step — admitted.
    SingleStep,
    /// Multi-step evaluation — NOT admitted without explicit Origin
    /// declaration structure (not yet defined through V0.9).
    /// Returns Err to structurally prevent evaluation.
    MultiStep { direction_basis: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InterventionPair {
    pub disturbance_node: usize,
    pub disturbance_delta: f64,
    pub rho_base: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InterventionComparison {
    pub primary_arm: ExperimentRecord,
    pub control_arm: ExperimentRecord,
    pub intervention: InterventionPair,
    pub comparison_note: String,
}

/// Single-step evaluation through the ProcessDirection gate (G8).
///
/// SingleStep: admitted — returns Ok(ExperimentRecord).
/// MultiStep: structurally blocked — returns Err.
///   Σ(Δ(x_r)) does not imply x_{r+1}. Multi-step evaluation
///   requires explicit Origin declaration of the sequential
///   observation sequence and relational evolution direction
///   (operators.rs V7 sequential observation requirement).
///   That declaration structure is not yet defined through V0.9.
///   Until it is, MultiStep returns Err unconditionally.
pub fn run_single_step(
    fixture_id: &str,
    x: &NodeField,
    rel: &DeclaredRelations,
    direction: &ProcessDirection,
    rho_base: f64,
) -> Result<ExperimentRecord, String> {
    match direction {
        ProcessDirection::SingleStep => {
            Ok(run_experiment(fixture_id, x, rel, rho_base))
        }
        ProcessDirection::MultiStep { direction_basis } => {
            Err(format!(
                "G8 ProcessDirection gate: MultiStep evaluation is not admitted \
                 without explicit Origin declaration of sequential observation \
                 sequence and relational evolution direction. \
                 Σ(Δ(x_r)) does not imply x_{{r+1}}. \
                 direction_basis='{}' is present but the required Origin \
                 declaration structure has not been defined through V0.9. \
                 Declare the sequential observation sequence through M before \
                 invoking MultiStep.",
                direction_basis
            ))
        }
    }
}

/// Matched intervention comparison (F8 vs F9).
/// x and x' constructed independently before either enters kernel (G9).
pub fn run_intervention_comparison(
    rel_primary: &DeclaredRelations,
    rel_control: &DeclaredRelations,
    base_values: Vec<f64>,
    disturbance_node: usize,
    disturbance_delta: f64,
    rho_base: f64,
) -> InterventionComparison {
    // G9: construct independently before either enters kernel
    let _x = NodeField::single(base_values.clone(), ObservationClass::SimulatedInput);
    let x_prime = NodeField::single(base_values.clone(), ObservationClass::SimulatedInput)
        .with_disturbance_at(disturbance_node, disturbance_delta);

    let primary_arm = run_experiment("F8_3LCT_intervention", &x_prime, rel_primary, rho_base);
    let control_arm = run_experiment("F9_chain_intervention", &x_prime, rel_control, rho_base);

    InterventionComparison {
        primary_arm,
        control_arm,
        intervention: InterventionPair { disturbance_node, disturbance_delta, rho_base },
        comparison_note:
            "Matched disturbance applied to 3LCT (F8) and chain (F9). \
             Outputs recorded for comparison. No predetermined result (G11)."
                .to_string(),
    }
}

/// F10: declared removal of C→A — topological change (binary, not scalar).
pub fn run_closure_removal(
    rel_closed: &DeclaredRelations,
    rel_open: &DeclaredRelations,
    values: Vec<f64>,
    rho_base: f64,
) -> (ExperimentRecord, ExperimentRecord) {
    let x = NodeField::single(values, ObservationClass::SimulatedInput);
    let rec_closed = run_experiment("F3_3LCT_full", &x, rel_closed, rho_base);
    let rec_open   = run_experiment("F10_closure_removed", &x, rel_open, rho_base);
    (rec_closed, rec_open)
}

// ── V0.2: Origin-declared input fields ───────────────────────────────────────
//
// Declared by Origin, 2026-09-20.
// k=1: ascending sequence for each node count.
// k=2: ascending sequence (component 0) + cyclic rotation (component 1).
// These are declared experimental inputs — no rank property is guaranteed
// or intended. The kernel observes whatever rank results.

/// Declared k=1 field for n nodes.
/// Panics if n is not in {2, 3, 4} — only declared node counts are admitted.
pub fn declared_field_k1(n: usize) -> NodeField {
    let values = match n {
        2 => vec![1.0, 2.0],
        3 => vec![1.0, 2.0, 3.0],
        4 => vec![1.0, 2.0, 3.0, 4.0],
        _ => panic!(
            "declared_field_k1: n={} is not in the Origin-declared set {{2,3,4}}. \
             Declare a field for this node count before passing it to V0.2 sweeps.",
            n
        ),
    };
    NodeField::single(values, ObservationClass::SimulatedInput)
}

/// Declared k=2 field for n nodes.
/// Component 0: ascending sequence. Component 1: cyclic rotation of component 0.
/// Panics if n is not in {2, 3, 4}.
pub fn declared_field_k2(n: usize) -> NodeField {
    let (c0, c1) = match n {
        2 => (vec![1.0, 2.0], vec![2.0, 1.0]),
        3 => (vec![1.0, 2.0, 3.0], vec![2.0, 3.0, 1.0]),
        4 => (vec![1.0, 2.0, 3.0, 4.0], vec![2.0, 3.0, 4.0, 1.0]),
        _ => panic!(
            "declared_field_k2: n={} is not in the Origin-declared set {{2,3,4}}. \
             Declare fields for this node count before passing to V0.2 sweeps.",
            n
        ),
    };
    NodeField::new(vec![c0, c1], ObservationClass::SimulatedInput)
}

// ── V0.2: Sweep functions ─────────────────────────────────────────────────────

/// k-matrix sweep over k ∈ {1,2} × rho_base_sweep for a single topology.
///
/// G14: produces exactly 2 × rho_base_sweep.len() records.
/// G15: k in each record equals x.n_components — confirmed by run_experiment
///      recording x.n_components directly as ExperimentRecord.k.
///
/// fixture_id_prefix is combined with k and rho_base to form the record id.
/// rho_base_sweep must be a non-empty slice within the declared range [0.1, 0.5].
pub fn sweep_k_matrix(
    fixture_id_prefix: &str,
    rel: &DeclaredRelations,
    rho_base_sweep: &[f64],
) -> Vec<ExperimentRecord> {
    assert!(!rho_base_sweep.is_empty(), "rho_base_sweep must be non-empty");
    let n = rel.n_nodes;
    let mut records = Vec::with_capacity(2 * rho_base_sweep.len());

    for k in [1usize, 2usize] {
        let x = if k == 1 {
            declared_field_k1(n)
        } else {
            declared_field_k2(n)
        };
        for &rb in rho_base_sweep {
            let id = format!("{}_k{}_rb{}", fixture_id_prefix, k, rb);
            records.push(run_experiment(&id, &x, rel, rb));
        }
    }

    // G14 check — structural, not just asserted in tests
    let expected = 2 * rho_base_sweep.len();
    assert_eq!(
        records.len(), expected,
        "G14: sweep_k_matrix produced {} records, expected {}",
        records.len(), expected
    );

    records
}

/// Intervention atlas sweep.
/// Applies each (disturbance_node, disturbance_delta, rho_base) triple to
/// both rel_primary (3LCT) and rel_control (chain), producing one
/// InterventionComparison per triple.
///
/// G16: primary_arm and control_arm topology labels must differ — enforced
///      by requiring rel_primary.label != rel_control.label at entry.
///
/// disturbance_nodes: declared set of loci to perturb.
/// disturbance_deltas: declared set of disturbance magnitudes.
/// rho_base_sweep: declared convention sweep.
/// base_values: Origin-declared k=1 field values for the shared node count.
pub fn sweep_intervention_atlas(
    rel_primary: &DeclaredRelations,
    rel_control: &DeclaredRelations,
    base_values: Vec<f64>,
    disturbance_nodes: &[usize],
    disturbance_deltas: &[f64],
    rho_base_sweep: &[f64],
) -> Vec<InterventionComparison> {
    // G16: topology labels must differ
    assert_ne!(
        rel_primary.label, rel_control.label,
        "G16: primary and control topologies must have different labels"
    );
    assert!(!disturbance_nodes.is_empty(), "disturbance_nodes must be non-empty");
    assert!(!disturbance_deltas.is_empty(), "disturbance_deltas must be non-empty");
    assert!(!rho_base_sweep.is_empty(), "rho_base_sweep must be non-empty");

    let mut comparisons = Vec::new();
    for &node in disturbance_nodes {
        for &delta in disturbance_deltas {
            for &rb in rho_base_sweep {
                comparisons.push(run_intervention_comparison(
                    rel_primary,
                    rel_control,
                    base_values.clone(),
                    node,
                    delta,
                    rb,
                ));
            }
        }
    }
    comparisons
}

/// Closure removal sweep: F3 (3LCT closed) vs F10 (C→A removed, open chain)
/// across k ∈ {1,2} × rho_base_sweep, using declared Origin input fields.
///
/// Returns one pair (closed_record, open_record) per (k, rho_base) combination.
pub fn sweep_closure_removal(
    rel_closed: &DeclaredRelations,
    rel_open: &DeclaredRelations,
    rho_base_sweep: &[f64],
) -> Vec<(ExperimentRecord, ExperimentRecord)> {
    assert!(!rho_base_sweep.is_empty(), "rho_base_sweep must be non-empty");
    // Both topologies must be n=3 — F3 and F10 are both three-locus
    assert_eq!(rel_closed.n_nodes, 3,
        "sweep_closure_removal: rel_closed must be n=3 (F3 3LCT)");
    assert_eq!(rel_open.n_nodes, 3,
        "sweep_closure_removal: rel_open must be n=3 (F10 chain)");

    let mut pairs = Vec::new();
    for k in [1usize, 2usize] {
        let x = if k == 1 {
            declared_field_k1(3)
        } else {
            declared_field_k2(3)
        };
        for &rb in rho_base_sweep {
            let id_closed = format!("F3_3LCT_full_k{}_rb{}", k, rb);
            let id_open   = format!("F10_closure_removed_k{}_rb{}", k, rb);
            let rec_closed = run_experiment(&id_closed, &x, rel_closed, rb);
            let rec_open   = run_experiment(&id_open,   &x, rel_open,   rb);
            pairs.push((rec_closed, rec_open));
        }
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::*;
    use crate::node_field::ObservationClass;

    #[test]
    fn g8_single_step_admitted() {
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], ObservationClass::SimulatedInput);
        let result = run_single_step("g8_pos_test", &x, &rel, &ProcessDirection::SingleStep, 0.3);
        assert!(result.is_ok(), "SingleStep must be admitted");
        assert!(result.unwrap().failure_mode.is_none());
    }

    #[test]
    fn g8_multi_step_blocked() {
        // Negative test: MultiStep must be structurally blocked (G8).
        // Σ(Δ(x_r)) does not imply x_{r+1}.
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], ObservationClass::SimulatedInput);
        let result = run_single_step(
            "g8_neg_test",
            &x,
            &rel,
            &ProcessDirection::MultiStep {
                direction_basis: "test_direction".to_string(),
            },
            0.3,
        );
        assert!(result.is_err(),
            "MultiStep must return Err — not admitted without Origin declaration (G8)");
        let err = result.unwrap_err();
        assert!(err.contains("G8 ProcessDirection gate"),
            "Error must identify G8 gate: {}", err);
    }

    #[test]
    fn g9_x_and_x_prime_independent() {
        let base = vec![1.0, 2.0, 3.0];
        let x = NodeField::single(base.clone(), ObservationClass::SimulatedInput);
        let x_prime = NodeField::single(base.clone(), ObservationClass::SimulatedInput)
            .with_disturbance_at(0, 0.5);
        assert!((x.field[0][0] - 1.0).abs() < 1e-12, "x must be unchanged");
        assert!((x_prime.field[0][0] - 1.5).abs() < 1e-12, "x' must have disturbance");
    }

    #[test]
    fn g11_both_arms_produce_output() {
        let comparison = run_intervention_comparison(
            &fixture_f8_closed_relational_3_intervention(),
            &fixture_f9_chain3_intervention(),
            vec![1.0, 2.0, 3.0], 0, 0.5, 0.3,
        );
        assert!(comparison.primary_arm.sigma_values.is_some());
        assert!(comparison.control_arm.sigma_values.is_some());
    }

    #[test]
    fn g13_record_produced_for_every_fixture() {
        let rel = fixture_f3_closed_relational_3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], ObservationClass::SimulatedInput);
        let rec = run_experiment("g13_test", &x, &rel, 0.3);
        assert!(!rec.fixture_id.is_empty());
        assert!(rec.rho_p.is_some());
        assert!((rec.rho_base - 0.3).abs() < 1e-12);
    }

    // ── V0.2 gate tests ───────────────────────────────────────────────────────

    #[test]
    fn declared_fields_k1_correct_values() {
        // Origin declaration: ascending sequence for each n.
        let f2 = declared_field_k1(2);
        assert_eq!(f2.n_components, 1);
        assert_eq!(f2.n_nodes, 2);
        assert!((f2.field[0][0] - 1.0).abs() < 1e-12);
        assert!((f2.field[0][1] - 2.0).abs() < 1e-12);

        let f3 = declared_field_k1(3);
        assert_eq!(f3.n_nodes, 3);
        assert!((f3.field[0][2] - 3.0).abs() < 1e-12);

        let f4 = declared_field_k1(4);
        assert_eq!(f4.n_nodes, 4);
        assert!((f4.field[0][3] - 4.0).abs() < 1e-12);
    }

    #[test]
    fn declared_fields_k2_correct_values() {
        // Origin declaration: component 0 = ascending, component 1 = cyclic rotation.
        let f3 = declared_field_k2(3);
        assert_eq!(f3.n_components, 2);
        assert_eq!(f3.n_nodes, 3);
        // component 0: [1,2,3]
        assert!((f3.field[0][0] - 1.0).abs() < 1e-12);
        assert!((f3.field[0][1] - 2.0).abs() < 1e-12);
        assert!((f3.field[0][2] - 3.0).abs() < 1e-12);
        // component 1: [2,3,1]
        assert!((f3.field[1][0] - 2.0).abs() < 1e-12);
        assert!((f3.field[1][1] - 3.0).abs() < 1e-12);
        assert!((f3.field[1][2] - 1.0).abs() < 1e-12);

        let f4 = declared_field_k2(4);
        assert_eq!(f4.n_components, 2);
        assert_eq!(f4.n_nodes, 4);
        // component 1: [2,3,4,1]
        assert!((f4.field[1][0] - 2.0).abs() < 1e-12);
        assert!((f4.field[1][3] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn declared_field_k1_rejects_undeclared_n() {
        // Only n ∈ {2,3,4} are declared — n=5 must panic.
        let result = std::panic::catch_unwind(|| declared_field_k1(5));
        assert!(result.is_err(), "n=5 must panic — not an Origin-declared node count");
    }

    #[test]
    fn declared_field_k2_rejects_undeclared_n() {
        let result = std::panic::catch_unwind(|| declared_field_k2(5));
        assert!(result.is_err(), "n=5 must panic — not an Origin-declared node count");
    }

    #[test]
    fn g14_sweep_k_matrix_record_count() {
        // G14: sweep_k_matrix produces exactly 2 × rho_base_sweep.len() records.
        use crate::kernel::RHO_BASE_SWEEP;
        let rel = fixture_f3_closed_relational_3();
        let records = sweep_k_matrix("F3_g14", &rel, &RHO_BASE_SWEEP);
        assert_eq!(records.len(), 2 * RHO_BASE_SWEEP.len(),
            "G14: record count must be 2 × rho_base_sweep.len()");
    }

    #[test]
    fn g15_k_in_record_equals_n_components() {
        // G15: k in each record must equal x.n_components.
        use crate::kernel::RHO_BASE_SWEEP;
        let rel = fixture_f2_chain3();
        let records = sweep_k_matrix("F2_g15", &rel, &RHO_BASE_SWEEP);
        // First RHO_BASE_SWEEP.len() records are k=1
        for rec in records.iter().take(RHO_BASE_SWEEP.len()) {
            assert_eq!(rec.k, 1, "G15: k=1 records must have k=1");
        }
        // Remaining records are k=2
        for rec in records.iter().skip(RHO_BASE_SWEEP.len()) {
            assert_eq!(rec.k, 2, "G15: k=2 records must have k=2");
        }
    }

    #[test]
    fn g16_intervention_atlas_labels_differ() {
        // G16: primary and control must have different topology labels.
        // Positive case: F8 (3LCT) vs F9 (Chain3) — must succeed.
        let result = std::panic::catch_unwind(|| {
            sweep_intervention_atlas(
                &fixture_f8_closed_relational_3_intervention(),
                &fixture_f9_chain3_intervention(),
                vec![1.0, 2.0, 3.0],
                &[0usize],
                &[0.5f64],
                &[0.3f64],
            )
        });
        assert!(result.is_ok(), "F8 vs F9 must be admitted (different labels)");

        // Negative case: F3 vs F3 — same label must panic.
        let result = std::panic::catch_unwind(|| {
            sweep_intervention_atlas(
                &fixture_f3_closed_relational_3(),
                &fixture_f3_closed_relational_3(),
                vec![1.0, 2.0, 3.0],
                &[0usize],
                &[0.5f64],
                &[0.3f64],
            )
        });
        assert!(result.is_err(), "G16: same topology label must panic");
    }

    #[test]
    fn intervention_atlas_record_count() {
        // Total = disturbance_nodes.len() × disturbance_deltas.len() × rho_base.len()
        let comparisons = sweep_intervention_atlas(
            &fixture_f8_closed_relational_3_intervention(),
            &fixture_f9_chain3_intervention(),
            vec![1.0, 2.0, 3.0],
            &[0usize, 1, 2],
            &[0.1f64, 0.5, 1.0, 2.0],
            &[0.1f64, 0.3, 0.5],
        );
        assert_eq!(comparisons.len(), 3 * 4 * 3,
            "atlas must produce nodes × deltas × rho_base records");
    }

    #[test]
    fn closure_removal_sweep_record_count() {
        use crate::kernel::RHO_BASE_SWEEP;
        let pairs = sweep_closure_removal(
            &fixture_f3_closed_relational_3(),
            &fixture_f10_closure_removed(),
            &RHO_BASE_SWEEP,
        );
        // 2 k values × 3 rho_base values = 6 pairs
        assert_eq!(pairs.len(), 2 * RHO_BASE_SWEEP.len(),
            "closure removal sweep must produce 2 × rho_base_sweep.len() pairs");
    }

    #[test]
    fn sweep_k_matrix_all_fixtures() {
        // G13 extension: every fixture in F1–F7 produces records from sweep.
        use crate::kernel::RHO_BASE_SWEEP;
        let fixtures: Vec<(&str, crate::topology::DeclaredRelations)> = vec![
            ("F1", fixture_f1_pair()),
            ("F2", fixture_f2_chain3()),
            ("F3", fixture_f3_closed_relational_3()),
            ("F4", fixture_f4_chain4()),
            ("F5", fixture_f5_branch()),
            ("F6", fixture_f6_convergent()),
            ("F7", fixture_f7_diamond()),
        ];
        for (id, rel) in &fixtures {
            let records = sweep_k_matrix(id, rel, &RHO_BASE_SWEEP);
            assert_eq!(records.len(), 2 * RHO_BASE_SWEEP.len(),
                "G14 failed for fixture {}", id);
            for rec in &records {
                assert!(rec.rho_p.is_some(),
                    "rho_p must be present for fixture {}", id);
                assert!(!rec.fixture_id.is_empty(),
                    "fixture_id must be present for fixture {}", id);
            }
        }
    }
}
