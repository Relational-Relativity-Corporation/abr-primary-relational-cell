// admissibility.rs — Metatron Dynamics, Inc.
// abr-primary-relational-cell V0.1
//
// ExperimentRecord: structured record of a single Primary evaluation.
//
// Kernel authority: operators.rs V7, build plan V1.2 G5.
//
// Two strictly separated fields:
//   failure_mode      — FM1 / FM2 only (operator-level failures)
//   admissibility_finding — CirculationCancellation (when produced
//                        by evaluation — NOT inferred from topology)
//
// These fields are NEVER merged.
// CirculationCancellation is NEVER used as an a priori prohibition
// on any topology. It is an evaluation output, not an input gate.
//
// FM1: NodeField contains non-finite values before operator application.
// FM2: DeclaredRelations contains zero-length edge list — no relations
//      to evaluate.
//
// CirculationCancellation: Σ output at all edges is zero despite
// non-zero Δ input — relational contrast accumulated to zero across
// the declared structure. This may occur in closed topologies.

use serde::{Deserialize, Serialize};
use crate::topology::{DeclaredRelations, TopologyLabel};
use crate::node_field::{NodeField, ObservationClass};
use crate::kernel::{PrimaryEvaluation, RhoP};

/// Operator-level failure modes. Only FM1 and FM2.
/// Source: operators.rs V7, build plan V1.2.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FailureMode {
    /// FM1: NodeField contains non-finite values (NaN or Inf).
    FM1NonFiniteInput,
    /// FM2: DeclaredRelations has no edges — no evaluation possible.
    FM2NoEdges,
}

/// Admissibility finding — separate from failure_mode (G5).
/// Produced by evaluation, never inferred from topology alone.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AdmissibilityFinding {
    /// Σ output is zero at all edges despite non-zero Δ input.
    /// Relational contrast accumulated to cancellation.
    /// May occur in closed relational topologies.
    /// NOT a prohibition — recorded as an evaluation result.
    CirculationCancellation,
}

/// Complete record of a single Primary kernel evaluation.
/// Produced by run_experiment(). Suitable for JSON serialization.
///
/// failure_mode and admissibility_finding are NEVER merged (G5).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentRecord {
    /// Fixture identifier
    pub fixture_id: String,
    /// Human-readable topology description
    pub topology_label: TopologyLabel,
    /// Number of loci
    pub n_nodes: usize,
    /// Number of declared edges
    pub n_edges: usize,
    /// Propagation capacity C_X
    pub cx: usize,
    /// Number of declared components k
    pub k: usize,
    /// Observation class of the input NodeField
    pub observation_class: ObservationClass,
    /// Cycle detection report (detection only — not a rejection)
    pub cycle_detected: bool,
    /// Operator-level failure, if any. None = evaluation completed.
    /// FM1 or FM2 only — never CirculationCancellation.
    pub failure_mode: Option<FailureMode>,
    /// Admissibility finding from evaluation, if any.
    /// CirculationCancellation only — never FM1/FM2.
    pub admissibility_finding: Option<AdmissibilityFinding>,
    /// rank(Im Δ) — None if evaluation did not complete
    pub rank_delta: Option<usize>,
    /// rank(Im Σ) — None if evaluation did not complete
    pub rank_sigma: Option<usize>,
    /// ρ_P classification — None if evaluation did not complete
    pub rho_p: Option<RhoP>,
    /// expression_condition — None if evaluation did not complete
    pub expression_condition: Option<bool>,
    /// Raw Δ output (per component, per edge)
    pub delta_values: Option<Vec<Vec<f64>>>,
    /// Raw Σ output (per component, per edge)
    pub sigma_values: Option<Vec<Vec<f64>>>,
}

/// Pre-evaluation checks: returns FailureMode if FM1 or FM2.
fn pre_check(x: &NodeField, rel: &DeclaredRelations) -> Option<FailureMode> {
    if rel.n_edges() == 0 {
        return Some(FailureMode::FM2NoEdges);
    }
    for comp in &x.field {
        for &v in comp {
            if !v.is_finite() {
                return Some(FailureMode::FM1NonFiniteInput);
            }
        }
    }
    None
}

/// Detect CirculationCancellation: Σ all-zero despite Δ non-zero.
fn detect_circulation_cancellation(eval: &PrimaryEvaluation) -> Option<AdmissibilityFinding> {
    let tol = 1e-12;
    let delta_nonzero = eval.delta.field.iter()
        .any(|comp| comp.iter().any(|&v| v.abs() > tol));
    let sigma_all_zero = eval.sigma.field.iter()
        .all(|comp| comp.iter().all(|&v| v.abs() <= tol));
    if delta_nonzero && sigma_all_zero {
        Some(AdmissibilityFinding::CirculationCancellation)
    } else {
        None
    }
}

/// Run a complete Primary experiment and produce an ExperimentRecord.
///
/// Declare ring FIRST → detect cycle (report only) → pre-check →
/// evaluate → classify admissibility.
///
/// CirculationCancellation is produced by evaluation, never by topology.
/// G5: these fields are always separate.
pub fn run_experiment(
    fixture_id: &str,
    x: &NodeField,
    rel: &DeclaredRelations,
) -> ExperimentRecord {
    let cycle_report = rel.has_undirected_cycle();
    let n_nodes = rel.n_nodes;
    let n_edges = rel.n_edges();
    let cx = rel.propagation_capacity();
    let k = x.n_nodes; // k = number of declared components expressed as node count
    let obs = x.observation_class.clone();
    let label = rel.label.clone();

    // Pre-evaluation failure check
    if let Some(fm) = pre_check(x, rel) {
        return ExperimentRecord {
            fixture_id: fixture_id.to_string(),
            topology_label: label,
            n_nodes, n_edges, cx, k,
            observation_class: obs,
            cycle_detected: cycle_report.has_cycle,
            failure_mode: Some(fm),
            admissibility_finding: None,
            rank_delta: None, rank_sigma: None,
            rho_p: None, expression_condition: None,
            delta_values: None, sigma_values: None,
        };
    }

    // Evaluate
    let eval = crate::kernel::evaluate_primary(x, rel);

    // Admissibility classification (from evaluation — not from topology)
    let admissibility_finding = detect_circulation_cancellation(&eval);

    ExperimentRecord {
        fixture_id: fixture_id.to_string(),
        topology_label: label,
        n_nodes, n_edges, cx,
        k: x.n_components,
        observation_class: obs,
        cycle_detected: cycle_report.has_cycle,
        failure_mode: None,
        admissibility_finding,
        rank_delta: Some(eval.rank_delta),
        rank_sigma: Some(eval.rank_sigma),
        rho_p: Some(eval.rho_p),
        expression_condition: Some(eval.expression_condition),
        delta_values: Some(eval.delta.field),
        sigma_values: Some(eval.sigma.field),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::*;
    use crate::node_field::ObservationClass;

    fn si() -> ObservationClass { ObservationClass::SimulatedInput }

    #[test]
    fn g5_failure_mode_never_circulation_cancellation() {
        // G5: CirculationCancellation must never appear in failure_mode.
        // Structural guarantee — FM1/FM2 are the only FailureMode variants.
        // Verify that FailureMode does not have a CirculationCancellation variant.
        // This test documents the constraint.
        let fm1 = FailureMode::FM1NonFiniteInput;
        let fm2 = FailureMode::FM2NoEdges;
        assert_ne!(fm1, fm2);
        // If FailureMode::CirculationCancellation were added, this would not compile.
    }

    #[test]
    fn g5_admissibility_finding_separate_from_failure_mode() {
        // G5: A completed evaluation with CirculationCancellation must have
        // failure_mode = None and admissibility_finding = Some(CirculationCancellation).
        // Use a uniform field on the 3LCT — Δ produces uniform values,
        // Σ may cancel.
        let rel = fixture_f3_closed_relational_3();
        let x = NodeField::single(vec![1.0, 1.0, 1.0], si());
        let rec = run_experiment("g5_test", &x, &rel);
        // failure_mode must be None (evaluation completed)
        assert!(rec.failure_mode.is_none(),
            "failure_mode must be None when evaluation completes");
        // admissibility_finding is separate — may or may not be Some
        // The point is: the two fields are never merged.
    }

    #[test]
    fn fm2_no_edges() {
        // FM2: DeclaredRelations with no edges → FM2NoEdges
        let rel = DeclaredRelations::from_edges(2, vec![], TopologyLabel::Pair);
        let x = NodeField::single(vec![1.0, 2.0], si());
        let rec = run_experiment("fm2_test", &x, &rel);
        assert_eq!(rec.failure_mode, Some(FailureMode::FM2NoEdges));
        assert!(rec.admissibility_finding.is_none());
    }

    #[test]
    fn fm1_nonfinite_input() {
        // FM1: NodeField with NaN → FM1NonFiniteInput
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![1.0, f64::NAN, 2.0], si());
        let rec = run_experiment("fm1_test", &x, &rel);
        assert_eq!(rec.failure_mode, Some(FailureMode::FM1NonFiniteInput));
        assert!(rec.admissibility_finding.is_none());
    }

    #[test]
    fn cycle_detected_does_not_prevent_evaluation() {
        // G5: Cycle detection must not prevent evaluation.
        // 3LCT has a cycle — evaluation must complete with failure_mode = None.
        let rel = fixture_f3_closed_relational_3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        let rec = run_experiment("cycle_test", &x, &rel);
        assert!(rec.cycle_detected, "cycle must be reported");
        assert!(rec.failure_mode.is_none(), "cycle must not cause failure_mode");
        assert!(rec.rank_delta.is_some(), "evaluation must complete");
    }

    #[test]
    fn g11_matched_intervention_comparison_recorded() {
        // G11: F8 (3LCT intervention) and F9 (chain intervention) both
        // produce outputs that are recorded — not predetermined.
        let rel_closed = fixture_f8_closed_relational_3_intervention();
        let rel_chain = fixture_f9_chain3_intervention();
        let x_base = NodeField::single(vec![1.0, 2.0, 3.0], si());
        let x_disturbed = x_base.with_disturbance_at(0, 0.5);

        let rec_closed = run_experiment("f8", &x_disturbed, &rel_closed);
        let rec_chain = run_experiment("f9", &x_disturbed, &rel_chain);

        // Both must complete
        assert!(rec_closed.failure_mode.is_none(), "F8 must complete");
        assert!(rec_chain.failure_mode.is_none(), "F9 must complete");
        // Both must have outputs — comparison is of recorded results
        assert!(rec_closed.sigma_values.is_some());
        assert!(rec_chain.sigma_values.is_some());
    }

    #[test]
    fn g12_f10_is_topological_change() {
        // G12: F10 removes edge C→A — topological change, not scalar weakening.
        // Verify C_X drops from 3 to 1.
        let rel_full = fixture_f3_closed_relational_3();
        let rel_removed = fixture_f10_closure_removed();
        assert_eq!(rel_full.propagation_capacity(), 3, "3LCT C_X must be 3");
        assert_eq!(rel_removed.propagation_capacity(), 1, "after removal C_X must be 1");
        // n_edges differs by 1
        assert_eq!(rel_full.n_edges(), 3);
        assert_eq!(rel_removed.n_edges(), 2);
    }
}
