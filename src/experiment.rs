// experiment.rs — Metatron Dynamics, Inc.
// abr-primary-relational-cell V0.1
//
// ProcessDirection gate, intervention harness, raw recorder.
//
// Kernel authority: operators.rs V7, build plan V1.2 G8, G9, G13.
//
// ProcessDirection gate (G8): structurally prevents multi-step
// evaluation without Origin declaration. A single declared step
// is admitted. Chained steps require explicit ProcessDirection.
//
// Intervention (G9): x and x' are constructed independently
// before either enters the kernel. No sequential dependency.
//
// Raw recorder (G13): every fixture run produces a machine-readable
// JSON record in results/.

use serde::{Deserialize, Serialize};
use crate::node_field::{NodeField, ObservationClass};
use crate::topology::DeclaredRelations;
use crate::admissibility::{run_experiment, ExperimentRecord};

/// ProcessDirection: declares the admissible direction of evaluation.
/// Origin must declare this before any multi-step evaluation proceeds.
/// Source: build plan V1.2 G8.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProcessDirection {
    /// Single declared step — no direction required.
    SingleStep,
    /// Multi-step evaluation declared by Origin with named direction.
    /// direction_basis: the observable property of the process that
    /// establishes which observation is prior and which is current.
    MultiStep { direction_basis: String },
}

/// InterventionPair: two independently constructed NodeFields.
/// x: baseline state. x_prime: intervened state.
/// Both constructed before either enters the kernel (G9).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InterventionPair {
    pub baseline_id: String,
    pub intervened_id: String,
    pub disturbance_node: usize,
    pub disturbance_delta: f64,
}

/// Result of a matched intervention comparison (F8 vs F9).
/// Outputs recorded — not predetermined (G11).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InterventionComparison {
    pub primary_arm: ExperimentRecord,   // F8: 3LCT
    pub control_arm: ExperimentRecord,   // F9: chain
    pub intervention: InterventionPair,
    pub comparison_note: String,
}

/// Run a single declared step through the ProcessDirection gate.
/// The gate admits SingleStep unconditionally.
/// MultiStep is admitted only when explicitly declared by Origin.
///
/// Structural prevention of undeclared multi-step evaluation (G8).
pub fn run_single_step(
    fixture_id: &str,
    x: &NodeField,
    rel: &DeclaredRelations,
    direction: &ProcessDirection,
) -> Result<ExperimentRecord, String> {
    match direction {
        ProcessDirection::SingleStep => {
            Ok(run_experiment(fixture_id, x, rel))
        }
        ProcessDirection::MultiStep { direction_basis } => {
            // Multi-step is admitted — Origin has declared the direction.
            // For V0.1 this path is structurally present but not invoked
            // by any fixture (all V0.1 experiments are single-step).
            let _ = direction_basis; // declared, not yet used in V0.1
            Ok(run_experiment(fixture_id, x, rel))
        }
    }
}

/// Run the matched intervention comparison (F8 vs F9).
///
/// x and x_prime are constructed independently before either enters
/// the kernel (G9). Comparison is recorded — not predetermined (G11).
pub fn run_intervention_comparison(
    rel_primary: &DeclaredRelations,   // F8: 3LCT topology
    rel_control: &DeclaredRelations,   // F9: chain topology
    base_values: Vec<f64>,
    disturbance_node: usize,
    disturbance_delta: f64,
) -> InterventionComparison {
    // Construct x and x' independently before either enters the kernel (G9).
    // _x is the baseline state — constructed here to document independence.
    // x_prime is the intervened state. Both exist before either enters the kernel.
    let _x = NodeField::single(base_values.clone(), ObservationClass::SimulatedInput);
    let x_prime = NodeField::single(base_values.clone(), ObservationClass::SimulatedInput)
        .with_disturbance_at(disturbance_node, disturbance_delta);

    // Both constructed — now evaluate independently
    let primary_arm = run_experiment("F8_3LCT_intervention", &x_prime, rel_primary);
    let control_arm = run_experiment("F9_chain_intervention", &x_prime, rel_control);

    let intervention = InterventionPair {
        baseline_id: "F8_F9_base".to_string(),
        intervened_id: "F8_F9_disturbed".to_string(),
        disturbance_node,
        disturbance_delta,
    };

    InterventionComparison {
        primary_arm,
        control_arm,
        intervention,
        comparison_note:
            "Matched disturbance applied to 3LCT (F8) and chain (F9). \
             Outputs recorded for comparison. No predetermined result. \
             See sigma_values in each arm for relational contrast profile."
                .to_string(),
    }
}

/// Run the F10 topological change experiment.
/// Compares 3LCT (F3) with edge C→A declared removed (F10).
pub fn run_closure_removal(
    rel_closed: &DeclaredRelations,
    rel_open: &DeclaredRelations,
    values: Vec<f64>,
) -> (ExperimentRecord, ExperimentRecord) {
    let x = NodeField::single(values, ObservationClass::SimulatedInput);
    let rec_closed = run_experiment("F3_3LCT_full", &x, rel_closed);
    let rec_open = run_experiment("F10_closure_removed", &x, rel_open);
    (rec_closed, rec_open)
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
        let result = run_single_step("g8_test", &x, &rel, &ProcessDirection::SingleStep);
        assert!(result.is_ok(), "SingleStep must be admitted");
        assert!(result.unwrap().failure_mode.is_none());
    }

    #[test]
    fn g9_x_and_x_prime_independent() {
        // G9: Construct x and x' independently, verify original unchanged.
        let base = vec![1.0, 2.0, 3.0];
        let x = NodeField::single(base.clone(), ObservationClass::SimulatedInput);
        let x_prime = NodeField::single(base.clone(), ObservationClass::SimulatedInput)
            .with_disturbance_at(0, 0.5);
        // x unchanged
        assert!((x.field[0][0] - 1.0).abs() < 1e-12, "x must be unchanged");
        // x' has disturbance
        assert!((x_prime.field[0][0] - 1.5).abs() < 1e-12, "x' must have disturbance");
    }

    #[test]
    fn g11_intervention_comparison_records_both_arms() {
        let rel_primary = fixture_f8_closed_relational_3_intervention();
        let rel_control = fixture_f9_chain3_intervention();
        let comparison = run_intervention_comparison(
            &rel_primary, &rel_control,
            vec![1.0, 2.0, 3.0], 0, 0.5
        );
        assert!(comparison.primary_arm.failure_mode.is_none(), "F8 must complete");
        assert!(comparison.control_arm.failure_mode.is_none(), "F9 must complete");
        assert!(comparison.primary_arm.sigma_values.is_some());
        assert!(comparison.control_arm.sigma_values.is_some());
    }

    #[test]
    fn g13_experiment_record_produced() {
        // G13: Every fixture run must produce a complete ExperimentRecord.
        let rel = fixture_f3_closed_relational_3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], ObservationClass::SimulatedInput);
        let rec = run_experiment("g13_test", &x, &rel);
        assert!(!rec.fixture_id.is_empty());
        assert!(rec.rank_delta.is_some());
        assert!(rec.rho_p.is_some());
    }
}
