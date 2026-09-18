// kernel.rs — Metatron Dynamics, Inc.
// abr-primary-relational-cell V0.1.2
//
// Primary kernel: E_primary = Σ(Δ(x))
//
// Kernel authority: operators.rs V7. Every formula independently implemented
// against that authority. Kernel files are reference only — not a dependency.
//
// rho_base declaration (Origin, 18 Sep 2026):
//   Admissible Primary Region operating range: [0.1, 0.5].
//   Rationale: rho_base must be large enough that antisymmetric term is
//   nonzero (expression_condition admissible) and small enough that ρ_P ≪ 1
//   (unambiguously Primary Region, below unknown ABR activation threshold
//   OC-ρP-1). Values above 0.5 risk approaching the transition region.
//   Values below 0.1 suppress the antisymmetric term to near-zero.
//   This range is a declared Origin parameter — not an imported default.
//   Convention fixtures run at 0.1, 0.3, and 0.5 to span the declared range.
//
// B absent. ABR operators absent. No persistence. No path accumulation.

use nalgebra::DMatrix;
use serde::{Deserialize, Serialize};
use crate::topology::DeclaredRelations;
use crate::node_field::NodeField;

pub const SVD_TOLERANCE: f64 = 1e-10;

/// Declared rho_base range for Primary Region operation.
/// Origin declaration, 18 Sep 2026.
pub const RHO_BASE_MIN: f64 = 0.1;
pub const RHO_BASE_MAX: f64 = 0.5;
/// Convention sweep values spanning the declared range.
pub const RHO_BASE_SWEEP: [f64; 3] = [0.1, 0.3, 0.5];

/// Validate that a rho_base value is within the declared Primary Region range.
/// Panics if out of range — an undeclared value must not silently enter.
pub fn assert_rho_base_in_range(rho_base: f64) {
    assert!(
        rho_base >= RHO_BASE_MIN && rho_base <= RHO_BASE_MAX,
        "rho_base={} is outside declared Primary Region range [{}, {}]. \
         Origin must declare a value within this range.",
        rho_base, RHO_BASE_MIN, RHO_BASE_MAX
    );
}

// ── Primary Edge Field ────────────────────────────────────────────────────

/// Output field over declared edges. field[component][edge].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrimaryEdgeField {
    pub field: Vec<Vec<f64>>,
    pub n_components: usize,
    pub n_edges: usize,
}

impl PrimaryEdgeField {
    pub fn new(field: Vec<Vec<f64>>, n_edges: usize) -> Self {
        let n_components = field.len();
        assert!(n_components > 0, "at least one component required");
        assert!(field.iter().all(|c| c.len() == n_edges),
            "all components must have length n_edges");
        PrimaryEdgeField { field, n_components, n_edges }
    }
}

// ── Operator Δ ────────────────────────────────────────────────────────────

/// Δ(x)[e] = x[source(e)] − x[target(e)] for each component.
/// Source: operators.rs V7 operator_delta.
pub fn operator_delta(x: &NodeField, rel: &DeclaredRelations) -> PrimaryEdgeField {
    assert_eq!(x.n_nodes, rel.n_nodes,
        "NodeField n_nodes must match DeclaredRelations n_nodes");
    let n_edges = rel.n_edges();
    let field: Vec<Vec<f64>> = (0..x.n_components)
        .map(|c| rel.edges.iter()
            .map(|&(s, t)| x.field[c][s] - x.field[c][t])
            .collect())
        .collect();
    PrimaryEdgeField::new(field, n_edges)
}

// ── ρ (Primary edge form) ─────────────────────────────────────────────────

/// compute_rho_primary: edge-form ρ, evaluated at source locus of each edge.
///
/// ρ[e] = rho_base · χ[source(e)] / (1 + χ[source(e)])
/// χ[s] = max{ |Δ(x)[e']| : e' incident to s } over all components.
///
/// rho_base must be within declared Primary Region range [0.1, 0.5].
/// Source: operators.rs V7 compute_rho_primary lines 474–490.
pub fn compute_rho_primary(
    delta_field: &PrimaryEdgeField,
    rel: &DeclaredRelations,
    rho_base: f64,
) -> Vec<f64> {
    assert_rho_base_in_range(rho_base);
    let mut node_incident: Vec<Vec<usize>> = vec![Vec::new(); rel.n_nodes];
    for (e, &(s, t)) in rel.edges.iter().enumerate() {
        node_incident[s].push(e);
        node_incident[t].push(e);
    }
    rel.edges.iter().map(|&(s, _)| {
        let chi = node_incident[s].iter()
            .flat_map(|&e| delta_field.field.iter().map(move |c| c[e].abs()))
            .fold(0.0_f64, f64::max);
        rho_base * chi / (1.0 + chi)
    }).collect()
}

// ── Operator Σ ────────────────────────────────────────────────────────────

/// Σ(Δ)[e] = Δ[e] + ρ[e] · (Σ_{f ∈ adj⁺(e)} Δ[f] − Σ_{p ∈ adj⁻(e)} Δ[p])
///
/// Source: operators.rs V7 operator_sigma lines 509–527.
pub fn operator_sigma(
    delta_field: &PrimaryEdgeField,
    rel: &DeclaredRelations,
    rho_base: f64,
) -> PrimaryEdgeField {
    assert_eq!(delta_field.n_edges, rel.n_edges(),
        "delta field and declared relations must have the same edge count");
    let rho = compute_rho_primary(delta_field, rel, rho_base);
    let field: Vec<Vec<f64>> = (0..delta_field.n_components).map(|c| {
        (0..rel.n_edges()).map(|e| {
            let forward: f64 = rel.adj_plus[e].iter()
                .map(|&f| delta_field.field[c][f]).sum();
            let backward: f64 = rel.adj_minus[e].iter()
                .map(|&p| delta_field.field[c][p]).sum();
            delta_field.field[c][e] + rho[e] * (forward - backward)
        }).collect()
    }).collect();
    PrimaryEdgeField::new(field, rel.n_edges())
}

// ── Antisymmetric term ────────────────────────────────────────────────────

/// antisymmetric_term[e] = ρ[e] · (Σ_{adj⁺} Δ[f] − Σ_{adj⁻} Δ[p])
/// Source: operators.rs V7 antisymmetric_term lines 538–555.
pub fn antisymmetric_term(
    delta_field: &PrimaryEdgeField,
    rel: &DeclaredRelations,
    rho_base: f64,
) -> PrimaryEdgeField {
    assert_eq!(delta_field.n_edges, rel.n_edges());
    let rho = compute_rho_primary(delta_field, rel, rho_base);
    let field: Vec<Vec<f64>> = (0..delta_field.n_components).map(|c| {
        (0..rel.n_edges()).map(|e| {
            let forward: f64 = rel.adj_plus[e].iter()
                .map(|&f| delta_field.field[c][f]).sum();
            let backward: f64 = rel.adj_minus[e].iter()
                .map(|&p| delta_field.field[c][p]).sum();
            rho[e] * (forward - backward)
        }).collect()
    }).collect();
    PrimaryEdgeField::new(field, rel.n_edges())
}

// ── Rank via SVD ──────────────────────────────────────────────────────────

/// rank(Im F) via SVD. Matrix is n_edges × n_components.
/// Source: operators.rs V7 im_delta_rank lines 580–594.
pub fn image_rank(f: &PrimaryEdgeField) -> usize {
    if f.n_edges == 0 || f.n_components == 0 { return 0; }
    let n_rows = f.n_edges;
    let n_cols = f.n_components;
    let data: Vec<f64> = (0..n_rows)
        .flat_map(|e| (0..n_cols).map(move |c| f.field[c][e]))
        .collect();
    let mat = DMatrix::from_row_slice(n_rows, n_cols, &data);
    let svd = mat.svd(false, false);
    svd.singular_values.iter().filter(|&&s| s > SVD_TOLERANCE).count()
}

// ── ρ_P ──────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RhoPUndefined {
    NoPropagationCapacity,
    NoDeclaredComponents,
}

/// ρ_P classification.
/// CeilingBounded: k/C_X < 1 — declaration-imposed ceiling.
/// Determined: k/C_X ≥ 1 — no declaration ceiling below 1.
/// Source: operators.rs V7 rho_p_ratio lines 690–718.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RhoP {
    Determined {
        value: f64,
        n_components: usize,
        propagation_capacity: usize,
    },
    CeilingBounded {
        value: f64,
        ceiling: f64,
        n_components: usize,
        propagation_capacity: usize,
    },
    Undefined { reason: RhoPUndefined },
}

pub fn rho_p_ratio(sigma_field: &PrimaryEdgeField, rel: &DeclaredRelations) -> RhoP {
    let c_x = rel.propagation_capacity();
    if c_x == 0 {
        return RhoP::Undefined { reason: RhoPUndefined::NoPropagationCapacity };
    }
    if sigma_field.n_components == 0 {
        return RhoP::Undefined { reason: RhoPUndefined::NoDeclaredComponents };
    }
    let rank_sigma = image_rank(sigma_field);
    let n_components = sigma_field.n_components;
    let value = rank_sigma as f64 / c_x as f64;
    let ceiling = n_components as f64 / c_x as f64;
    if ceiling < 1.0 {
        RhoP::CeilingBounded { value, ceiling, n_components, propagation_capacity: c_x }
    } else {
        RhoP::Determined { value, n_components, propagation_capacity: c_x }
    }
}

// ── Expression condition ──────────────────────────────────────────────────

/// rank(Im Σ) > 0 AND antisymmetric term nonzero on at least one edge.
/// Source: operators.rs V7 expression_condition lines 748–759.
pub fn expression_condition(
    delta_field: &PrimaryEdgeField,
    sigma_field: &PrimaryEdgeField,
    rel: &DeclaredRelations,
    rho_base: f64,
) -> (bool, usize, bool) {
    let rank_sigma = image_rank(sigma_field);
    let asym = antisymmetric_term(delta_field, rel, rho_base);
    let has_nonzero_asym = asym.field.iter()
        .any(|c| c.iter().any(|&v| v.abs() > SVD_TOLERANCE));
    let expressed = rank_sigma > 0 && has_nonzero_asym;
    (expressed, rank_sigma, has_nonzero_asym)
}

// ── Failure modes ─────────────────────────────────────────────────────────

/// FM1 DifferentiationCollapse: rank(Im Δ) = 0.
/// FM2 RelationalIsolation: all edges isolated.
/// CirculationCancellation routed to admissibility_finding (V1.2 G5).
/// Source: operators.rs V7 lines 764–793.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FailureMode {
    DifferentiationCollapse,
    RelationalIsolation,
}

pub fn detect_failure_mode(
    delta_field: &PrimaryEdgeField,
    rel: &DeclaredRelations,
) -> Option<FailureMode> {
    if image_rank(delta_field) == 0 {
        return Some(FailureMode::DifferentiationCollapse);
    }
    let all_isolated = rel.adj_plus.iter().zip(rel.adj_minus.iter())
        .all(|(p, m)| p.is_empty() && m.is_empty());
    if all_isolated {
        return Some(FailureMode::RelationalIsolation);
    }
    None
}

// ── Full Primary evaluation ───────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrimaryEvaluation {
    pub delta: PrimaryEdgeField,
    pub sigma: PrimaryEdgeField,
    pub antisym: PrimaryEdgeField,
    pub rank_delta: usize,
    pub rank_sigma: usize,
    pub cx: usize,
    pub rho_p: RhoP,
    pub expression_condition: bool,
    pub rank_sigma_for_expr: usize,
    pub has_nonzero_antisym: bool,
    pub failure_mode: Option<FailureMode>,
    /// rho_base value used — within declared range [0.1, 0.5]
    pub rho_base: f64,
}

pub fn evaluate_primary(
    x: &NodeField,
    rel: &DeclaredRelations,
    rho_base: f64,
) -> PrimaryEvaluation {
    assert_rho_base_in_range(rho_base);
    let delta = operator_delta(x, rel);
    let failure_mode = detect_failure_mode(&delta, rel);
    let sigma = operator_sigma(&delta, rel, rho_base);
    let antisym = antisymmetric_term(&delta, rel, rho_base);
    let rank_delta = image_rank(&delta);
    let rank_sigma = image_rank(&sigma);
    let cx = rel.propagation_capacity();
    let rho_p = rho_p_ratio(&sigma, rel);
    let (expr, rs_expr, has_asym) = expression_condition(&delta, &sigma, rel, rho_base);
    PrimaryEvaluation {
        delta, sigma, antisym,
        rank_delta, rank_sigma, cx,
        rho_p,
        expression_condition: expr,
        rank_sigma_for_expr: rs_expr,
        has_nonzero_antisym: has_asym,
        failure_mode,
        rho_base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::*;
    use crate::node_field::ObservationClass;

    fn si() -> ObservationClass { ObservationClass::SimulatedInput }

    // Hand-calculated canonical values for chain3 x=[1,2,3] at rho_base=0.3:
    // Δ[e0]=−1, Δ[e1]=−1
    // χ[0]=1, χ[1]=1, χ[2]=1 → ρ[e0]=ρ[e1]=0.15
    // Σ[e0]=−1+0.15·(−1−0)=−1.15
    // Σ[e1]=−1+0.15·(0−(−1))=−0.85
    // antisym[e0]=0.15·(−1−0)=−0.15
    // antisym[e1]=0.15·(0−(−1))=+0.15

    #[test]
    fn canonical_delta_chain3() {
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        let d = operator_delta(&x, &rel);
        assert!((d.field[0][0] - (-1.0)).abs() < 1e-10, "Δ[e0]");
        assert!((d.field[0][1] - (-1.0)).abs() < 1e-10, "Δ[e1]");
    }

    #[test]
    fn canonical_rho_chain3_at_03() {
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        let d = operator_delta(&x, &rel);
        let rho = compute_rho_primary(&d, &rel, 0.3);
        assert!((rho[0] - 0.15).abs() < 1e-10, "ρ[e0]");
        assert!((rho[1] - 0.15).abs() < 1e-10, "ρ[e1]");
    }

    #[test]
    fn canonical_sigma_chain3_at_03() {
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        let d = operator_delta(&x, &rel);
        let s = operator_sigma(&d, &rel, 0.3);
        assert!((s.field[0][0] - (-1.15)).abs() < 1e-10, "Σ[e0]");
        assert!((s.field[0][1] - (-0.85)).abs() < 1e-10, "Σ[e1]");
    }

    #[test]
    fn canonical_antisym_chain3_at_03() {
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        let d = operator_delta(&x, &rel);
        let asym = antisymmetric_term(&d, &rel, 0.3);
        assert!((asym.field[0][0] - (-0.15)).abs() < 1e-10, "antisym[e0]");
        assert!((asym.field[0][1] - 0.15).abs()  < 1e-10, "antisym[e1]");
    }

    #[test]
    fn rho_base_range_enforcement() {
        // Values outside [0.1, 0.5] must panic
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        let d = operator_delta(&x, &rel);
        assert!(std::panic::catch_unwind(|| {
            compute_rho_primary(&d, &rel, 0.0)
        }).is_err(), "rho_base=0.0 must be rejected");
        assert!(std::panic::catch_unwind(|| {
            compute_rho_primary(&d, &rel, 0.6)
        }).is_err(), "rho_base=0.6 must be rejected");
    }

    #[test]
    fn rho_base_boundary_values_admitted() {
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        let d = operator_delta(&x, &rel);
        // 0.1 and 0.5 are within range — must not panic
        let _ = compute_rho_primary(&d, &rel, 0.1);
        let _ = compute_rho_primary(&d, &rel, 0.5);
    }

    #[test]
    fn g6_rho_p_pair_undefined() {
        let rel = fixture_f1_pair();
        let x = NodeField::single(vec![1.0, 2.0], si());
        let eval = evaluate_primary(&x, &rel, 0.3);
        assert!(matches!(eval.rho_p,
            RhoP::Undefined { reason: RhoPUndefined::NoPropagationCapacity }));
    }

    #[test]
    fn g6_rho_p_chain3_k1_determined() {
        // k=1, C_X=1 → ceiling=1.0 → Determined
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        let eval = evaluate_primary(&x, &rel, 0.3);
        assert!(matches!(eval.rho_p, RhoP::Determined { .. }));
    }

    #[test]
    fn g6_rho_p_3lct_k1_ceiling_bounded() {
        // k=1, C_X=3 → ceiling=0.333 → CeilingBounded
        let rel = fixture_f3_closed_relational_3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        let eval = evaluate_primary(&x, &rel, 0.3);
        assert!(matches!(eval.rho_p, RhoP::CeilingBounded { .. }));
    }

    #[test]
    fn g6_rho_p_3lct_k3_determined() {
        // k=3, C_X=3 → ceiling=1.0 → Determined (linearly independent fields)
        let rel = fixture_f3_closed_relational_3();
        let x = NodeField::new(vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 1.0, 2.0],
            vec![3.0, 5.0, 1.0],
        ], si());
        let eval = evaluate_primary(&x, &rel, 0.3);
        assert!(matches!(eval.rho_p, RhoP::Determined { .. }));
    }

    #[test]
    fn fm1_differentiation_collapse() {
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![5.0, 5.0, 5.0], si());
        let eval = evaluate_primary(&x, &rel, 0.3);
        assert_eq!(eval.failure_mode, Some(FailureMode::DifferentiationCollapse));
    }

    #[test]
    fn fm2_relational_isolation() {
        // PRC-2: adj_plus[e0]=[], adj_minus[e0]=[] → RelationalIsolation
        let rel = fixture_f1_pair();
        let x = NodeField::single(vec![1.0, 2.0], si());
        let eval = evaluate_primary(&x, &rel, 0.3);
        assert_eq!(eval.failure_mode, Some(FailureMode::RelationalIsolation));
    }

    #[test]
    fn expression_condition_false_for_isolated_edge() {
        let rel = fixture_f1_pair();
        let x = NodeField::single(vec![1.0, 2.0], si());
        let eval = evaluate_primary(&x, &rel, 0.3);
        assert!(!eval.has_nonzero_antisym);
        assert!(!eval.expression_condition);
    }

    #[test]
    fn expression_condition_true_chain3() {
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        let eval = evaluate_primary(&x, &rel, 0.3);
        assert!(eval.has_nonzero_antisym);
        assert!(eval.expression_condition);
    }

    #[test]
    fn rho_base_recorded_in_evaluation() {
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        for &rb in &RHO_BASE_SWEEP {
            let eval = evaluate_primary(&x, &rel, rb);
            assert!((eval.rho_base - rb).abs() < 1e-12,
                "rho_base must be recorded in evaluation output");
        }
    }

    #[test]
    fn all_outputs_finite_across_rho_base_sweep() {
        for &rb in &RHO_BASE_SWEEP {
            for rel in [
                fixture_f1_pair(),
                fixture_f2_chain3(),
                fixture_f3_closed_relational_3(),
                fixture_f4_chain4(),
                fixture_f5_branch(),
                fixture_f6_convergent(),
                fixture_f7_diamond(),
            ] {
                let n = rel.n_nodes;
                let x = NodeField::single(
                    (0..n).map(|i| (i + 1) as f64).collect(), si()
                );
                let eval = evaluate_primary(&x, &rel, rb);
                for c in 0..eval.delta.n_components {
                    assert!(eval.delta.field[c].iter().all(|v| v.is_finite()));
                    assert!(eval.sigma.field[c].iter().all(|v| v.is_finite()));
                    assert!(eval.antisym.field[c].iter().all(|v| v.is_finite()));
                }
            }
        }
    }
}
