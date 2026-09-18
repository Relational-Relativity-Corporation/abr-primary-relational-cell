// admissibility.rs — Metatron Dynamics, Inc.
// abr-primary-relational-cell V0.1.2
//
// rho_base is now an explicit Origin-declared parameter passed through
// every evaluation path. No default value is used anywhere.

use serde::{Deserialize, Serialize};
use crate::topology::{DeclaredRelations, TopologyLabel};
use crate::node_field::{NodeField, ObservationClass};
use crate::kernel::{
    FailureMode, PrimaryEdgeField, RhoP, SVD_TOLERANCE,
    evaluate_primary,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AdmissibilityFinding {
    /// Antisymmetric term zero on all edges despite nonzero Δ.
    /// Recorded from evaluation — never inferred from topology (G5).
    CirculationCancellation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentRecord {
    pub fixture_id: String,
    pub topology_label: TopologyLabel,
    pub n_nodes: usize,
    pub n_edges: usize,
    pub cx: usize,
    pub k: usize,
    pub observation_class: ObservationClass,
    pub cycle_detected: bool,
    /// Origin-declared rho_base used for this evaluation
    pub rho_base: f64,
    /// FM1/FM2 only — never CirculationCancellation (G5)
    pub failure_mode: Option<FailureMode>,
    /// CirculationCancellation only — never FM1/FM2 (G5)
    pub admissibility_finding: Option<AdmissibilityFinding>,
    pub rank_delta: Option<usize>,
    pub rank_sigma: Option<usize>,
    pub rho_p: Option<RhoP>,
    pub expression_condition: Option<bool>,
    pub rank_sigma_for_expr: Option<usize>,
    pub has_nonzero_antisym: Option<bool>,
    pub delta_values: Option<Vec<Vec<f64>>>,
    pub sigma_values: Option<Vec<Vec<f64>>>,
    pub antisym_values: Option<Vec<Vec<f64>>>,
}

fn detect_circulation_cancellation(
    antisym: &PrimaryEdgeField,
) -> Option<AdmissibilityFinding> {
    let all_zero = antisym.field.iter()
        .all(|c| c.iter().all(|&v| v.abs() <= SVD_TOLERANCE));
    if all_zero { Some(AdmissibilityFinding::CirculationCancellation) } else { None }
}

/// Run a complete Primary experiment at a declared rho_base value.
/// rho_base must be within Origin-declared range [0.1, 0.5].
pub fn run_experiment(
    fixture_id: &str,
    x: &NodeField,
    rel: &DeclaredRelations,
    rho_base: f64,
) -> ExperimentRecord {
    let cycle_report = rel.has_undirected_cycle();
    let n_nodes = rel.n_nodes;
    let n_edges = rel.n_edges();
    let cx = rel.propagation_capacity();
    let k = x.n_components;
    let obs = x.observation_class.clone();
    let label = rel.label.clone();

    let eval = evaluate_primary(x, rel, rho_base);
    let failure_mode = eval.failure_mode.clone();
    let admissibility_finding = detect_circulation_cancellation(&eval.antisym);

    ExperimentRecord {
        fixture_id: fixture_id.to_string(),
        topology_label: label,
        n_nodes, n_edges, cx, k,
        observation_class: obs,
        cycle_detected: cycle_report.has_cycle,
        rho_base,
        failure_mode,
        admissibility_finding,
        rank_delta: Some(eval.rank_delta),
        rank_sigma: Some(eval.rank_sigma),
        rho_p: Some(eval.rho_p),
        expression_condition: Some(eval.expression_condition),
        rank_sigma_for_expr: Some(eval.rank_sigma_for_expr),
        has_nonzero_antisym: Some(eval.has_nonzero_antisym),
        delta_values: Some(eval.delta.field),
        sigma_values: Some(eval.sigma.field),
        antisym_values: Some(eval.antisym.field),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::*;
    use crate::node_field::ObservationClass;
    use crate::kernel::FailureMode;

    fn si() -> ObservationClass { ObservationClass::SimulatedInput }

    #[test]
    fn g5_failure_mode_variants_fm1_fm2_only() {
        let fm1 = FailureMode::DifferentiationCollapse;
        let fm2 = FailureMode::RelationalIsolation;
        assert_ne!(fm1, fm2);
    }

    #[test]
    fn k_field_is_n_components() {
        let rel = fixture_f2_chain3();
        let x = NodeField::new(vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 1.0, 2.0],
        ], si());
        let rec = run_experiment("k_test", &x, &rel, 0.3);
        assert_eq!(rec.k, 2, "k must be x.n_components");
    }

    #[test]
    fn rho_base_recorded_in_record() {
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        for &rb in &[0.1f64, 0.3, 0.5] {
            let rec = run_experiment("rb_test", &x, &rel, rb);
            assert!((rec.rho_base - rb).abs() < 1e-12);
        }
    }

    #[test]
    fn cycle_does_not_prevent_evaluation() {
        let rel = fixture_f3_closed_relational_3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        let rec = run_experiment("cycle_test", &x, &rel, 0.3);
        assert!(rec.cycle_detected);
        assert!(rec.rank_delta.is_some());
    }

    #[test]
    fn g5_cc_in_admissibility_finding_not_failure_mode() {
        let rel = fixture_f3_closed_relational_3();
        let x = NodeField::single(vec![1.0, 1.0, 1.0], si());
        let rec = run_experiment("g5_test", &x, &rel, 0.3);
        if let Some(ref fm) = rec.failure_mode {
            assert!(matches!(fm,
                FailureMode::DifferentiationCollapse | FailureMode::RelationalIsolation
            ));
        }
    }

    #[test]
    fn g12_f10_topological_change() {
        assert_eq!(fixture_f3_closed_relational_3().propagation_capacity(), 3);
        assert_eq!(fixture_f10_closure_removed().propagation_capacity(), 1);
    }
}
