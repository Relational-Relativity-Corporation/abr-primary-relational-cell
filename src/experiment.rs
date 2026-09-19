// experiment.rs — Metatron Dynamics, Inc.
// abr-primary-relational-cell V0.1.3
//
// ProcessDirection gate (G8): MultiStep is structurally blocked.
// Σ(Δ(x_r)) does not imply x_{r+1}. Multi-step evaluation requires
// explicit Origin declaration — returning Err until that structure
// is declared and authorized. V0.1 operates on single declared steps only.

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
}
