// node_field.rs — Metatron Dynamics, Inc.
// abr-primary-relational-cell V0.1
//
// NodeField: declared observable components at each locus.
// ObservationClass: attached to every NodeField — tracks provenance
//   of the input observation.
//
// Kernel authority: operators.rs V7 lines 204–230 (NodeField),
//   build plan V1.2 G7 (k=0 rejection).
//
// k=0 is rejected at the NodeField constructor before any operator acts.
// ObservationClass distinguishes simulation input from transducer output
// from physical measurement — these are not interchangeable.

use serde::{Deserialize, Serialize};

/// Observation class attached to every NodeField.
/// Distinguishes provenance of the declared input.
/// Source: build plan V1.2 node_field.rs specification.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ObservationClass {
    /// Declared simulation input — values supplied directly by Origin
    /// for the purpose of computing operator outputs.
    SimulatedInput,
    /// Output of a declared transducer applied to a prior observable.
    /// ProcessDirection gate applies before this class is admitted
    /// as input to a subsequent step.
    SimulatedTransducerOutput,
    /// Physical measurement through declared mapping M.
    /// Required for physical correspondence (Stage 5+).
    PhysicalMeasurement,
}

/// Declared observable components at each locus.
/// field[c][v] = component c at node v.
/// k = field[0].len() (number of nodes); components = field.len().
///
/// k=0 is rejected at construction (G7).
/// Every NodeField carries an ObservationClass.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeField {
    /// field[component][node] = declared value.
    pub field: Vec<Vec<f64>>,
    /// Number of declared components.
    pub n_components: usize,
    /// Number of declared loci (nodes).
    pub n_nodes: usize,
    /// Observation provenance.
    pub observation_class: ObservationClass,
}

impl NodeField {
    /// Construct a NodeField from component vectors.
    /// Panics if:
    ///   - field is empty (no components)
    ///   - any component vector is empty (k=0 — G7)
    ///   - component vectors have unequal lengths
    pub fn new(field: Vec<Vec<f64>>, observation_class: ObservationClass) -> Self {
        assert!(!field.is_empty(), "NodeField must have at least one component");
        let n_nodes = field[0].len();
        assert!(
            n_nodes > 0,
            "NodeField k=0 is inadmissible — at least one declared component required (G7)"
        );
        for (c, comp) in field.iter().enumerate() {
            assert_eq!(
                comp.len(), n_nodes,
                "Component {} length {} differs from n_nodes {}",
                c, comp.len(), n_nodes
            );
        }
        let n_components = field.len();
        NodeField { field, n_components, n_nodes, observation_class }
    }

    /// Single-component constructor — most common case in Primary Region.
    pub fn single(values: Vec<f64>, observation_class: ObservationClass) -> Self {
        assert!(!values.is_empty(), "NodeField k=0 is inadmissible (G7)");
        let n_nodes = values.len();
        NodeField {
            field: vec![values],
            n_components: 1,
            n_nodes,
            observation_class,
        }
    }

    /// Apply a declared disturbance at a single locus.
    /// Returns a new NodeField (x') constructed independently from x.
    /// Both x and x' are constructed before either enters the kernel (G9).
    pub fn with_disturbance_at(&self, node: usize, delta: f64) -> Self {
        assert!(node < self.n_nodes, "disturbance node {} out of range", node);
        let mut new_field = self.field.clone();
        for comp in new_field.iter_mut() {
            comp[node] += delta;
        }
        NodeField {
            field: new_field,
            n_components: self.n_components,
            n_nodes: self.n_nodes,
            observation_class: ObservationClass::SimulatedInput,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn g7_k_zero_rejected() {
        // G7: k=0 must be rejected at constructor.
        let result = std::panic::catch_unwind(|| {
            NodeField::new(vec![vec![]], ObservationClass::SimulatedInput)
        });
        assert!(result.is_err(), "k=0 must panic at NodeField constructor (G7)");
    }

    #[test]
    fn single_component_constructs() {
        let nf = NodeField::single(vec![1.0, 2.0, 3.0], ObservationClass::SimulatedInput);
        assert_eq!(nf.n_nodes, 3);
        assert_eq!(nf.n_components, 1);
    }

    #[test]
    fn disturbance_constructs_independently() {
        // G9: x and x' constructed independently.
        let x = NodeField::single(vec![1.0, 2.0, 3.0], ObservationClass::SimulatedInput);
        let x_prime = x.with_disturbance_at(0, 0.5);
        // x unchanged
        assert!((x.field[0][0] - 1.0).abs() < 1e-12);
        // x' has disturbance
        assert!((x_prime.field[0][0] - 1.5).abs() < 1e-12);
        // other nodes unchanged
        assert!((x_prime.field[0][1] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn observation_class_preserved() {
        let nf = NodeField::single(vec![1.0, 2.0], ObservationClass::PhysicalMeasurement);
        assert_eq!(nf.observation_class, ObservationClass::PhysicalMeasurement);
    }
}
