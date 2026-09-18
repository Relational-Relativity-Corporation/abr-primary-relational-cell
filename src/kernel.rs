// kernel.rs — Metatron Dynamics, Inc.
// abr-primary-relational-cell V0.1
//
// Primary kernel: E_primary = Σ(Δ(x))
//
// Independently implements Δ, Σ, antisymmetric terms, rank(Im Δ),
// rank(Im Σ), propagation capacity C_X, ρ_P, expression_condition.
//
// Kernel authority: operators.rs V7. Every formula here is independently
// derived from that authority. Kernel files are reference only —
// not a code dependency.
//
// B absent. ABR operators absent. No persistence. No path accumulation.
// No model-generated trajectory. No statistical quantities.
// No quantum-mechanical primitives imported — consequences only through M.

use serde::{Deserialize, Serialize};
use crate::topology::DeclaredRelations;
use crate::node_field::NodeField;

// ── Primary Edge Field ────────────────────────────────────────────────────

/// Output of Δ applied to a NodeField over DeclaredRelations.
/// field[component][edge] = directed difference at that edge.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrimaryEdgeField {
    pub field: Vec<Vec<f64>>,
    pub n_components: usize,
    pub n_edges: usize,
}

/// Output of Σ applied to a PrimaryEdgeField over DeclaredRelations.
/// field[component][edge] = accumulated relational contrast at that edge.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SigmaField {
    pub field: Vec<Vec<f64>>,
    pub n_components: usize,
    pub n_edges: usize,
}

// ── ρ_P Classification ────────────────────────────────────────────────────

/// Named reason for ρ_P Undefined.
/// Source: operators.rs V7 — RhoPUndefined variants.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RhoPUndefinedReason {
    /// C_X = 0: no edge has a non-empty adj_plus. ρ_P has no denominator.
    NoDeclaredComponents,
    /// rank(Im Σ) could not be computed (degenerate field).
    DegenerateField,
}

/// ρ_P classification: ratio of rank(Im Σ) to propagation capacity C_X.
/// Source: operators.rs V7 lines 601–690.
///
/// Ceiling condition (G6): Determined requires k ≥ C_X.
/// CeilingBounded: k < C_X — rank is bounded by k, not C_X.
/// Undefined: C_X = 0 or degenerate field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RhoP {
    /// rank(Im Σ) ≥ C_X — propagation capacity fully expressed.
    Determined { rank_sigma: usize, cx: usize },
    /// rank(Im Σ) < C_X — ceiling imposed by declared components k.
    CeilingBounded { rank_sigma: usize, cx: usize },
    /// C_X = 0 or degenerate: ρ_P not defined.
    Undefined { reason: RhoPUndefinedReason },
}

// ── Operator Δ ────────────────────────────────────────────────────────────

/// Δ(x): directed difference at each declared edge.
/// Δ(x)[e] = x[source(e)] - x[target(e)] for each component.
///
/// Formula: operators.rs V7 operator_delta.
/// Independent implementation — kernel file is reference only.
pub fn operator_delta(x: &NodeField, rel: &DeclaredRelations) -> PrimaryEdgeField {
    assert_eq!(
        x.n_nodes, rel.n_nodes,
        "NodeField n_nodes must match DeclaredRelations n_nodes"
    );
    let n_edges = rel.n_edges();
    let mut field = vec![vec![0.0f64; n_edges]; x.n_components];
    for c in 0..x.n_components {
        for (e, &(s, t)) in rel.edges.iter().enumerate() {
            field[c][e] = x.field[c][s] - x.field[c][t];
        }
    }
    PrimaryEdgeField { field, n_components: x.n_components, n_edges }
}

// ── Operator Σ ────────────────────────────────────────────────────────────

/// Σ(d): relational contrast accumulated over adj_plus at each edge.
/// For each edge e: Σ(d)[e] = d[e] + Σ_{f ∈ adj_plus[e]} d[f]
///   minus the antisymmetric term: - Σ_{f ∈ adj_plus[e]} d[f] (signed reversal)
///
/// Full expression:
///   Σ(d)[e] = d[e] - Σ_{f ∈ adj_plus[e]} (d[f])
///
/// This is the antisymmetric accumulation: the edge's own contrast minus
/// the sum of its successors' contrasts. When adj_plus[e] is empty
/// (no successors), Σ(d)[e] = d[e].
///
/// Formula: operators.rs V7 operator_sigma.
/// Independent implementation — kernel file is reference only.
pub fn operator_sigma(d: &PrimaryEdgeField, rel: &DeclaredRelations) -> SigmaField {
    assert_eq!(d.n_edges, rel.n_edges(), "PrimaryEdgeField n_edges must match DeclaredRelations");
    let n_edges = d.n_edges;
    let mut field = vec![vec![0.0f64; n_edges]; d.n_components];
    for c in 0..d.n_components {
        for e in 0..n_edges {
            let self_term = d.field[c][e];
            let successor_sum: f64 = rel.adj_plus[e].iter().map(|&f| d.field[c][f]).sum();
            field[c][e] = self_term - successor_sum;
        }
    }
    SigmaField { field, n_components: d.n_components, n_edges }
}

// ── Rank computation ──────────────────────────────────────────────────────

/// Compute rank of a matrix given as rows (each row is a Vec<f64>).
/// Uses Gaussian elimination with column-searching partial pivoting.
/// For each row, scans all remaining columns to find a pivot —
/// correctly handles leading zeros in any row.
/// Tolerance: values below tol treated as zero.
///
/// Used for rank(Im Δ) and rank(Im Σ).
pub fn matrix_rank(rows: &[Vec<f64>], tol: f64) -> usize {
    if rows.is_empty() { return 0; }
    let n_cols = rows[0].len();
    if n_cols == 0 { return 0; }

    let mut mat: Vec<Vec<f64>> = rows.to_vec();
    let n_rows = mat.len();
    let mut rank = 0;
    let mut pivot_col = 0;

    for row in 0..n_rows {
        // Scan columns from pivot_col onward for a non-zero entry in rows [row..]
        let mut found = false;
        while pivot_col < n_cols && !found {
            // Find the row with largest absolute value in this column
            let mut max_row = row;
            let mut max_val = mat[row][pivot_col].abs();
            for r in (row + 1)..n_rows {
                if mat[r][pivot_col].abs() > max_val {
                    max_val = mat[r][pivot_col].abs();
                    max_row = r;
                }
            }
            if max_val < tol {
                // No pivot in this column — advance to next column
                pivot_col += 1;
            } else {
                mat.swap(row, max_row);
                let pivot = mat[row][pivot_col];
                for v in mat[row].iter_mut() { *v /= pivot; }
                for r in 0..n_rows {
                    if r != row {
                        let factor = mat[r][pivot_col];
                        for c in 0..n_cols {
                            let sub = factor * mat[row][c];
                            mat[r][c] -= sub;
                        }
                    }
                }
                rank += 1;
                pivot_col += 1;
                found = true;
            }
        }
        if !found { break; }
    }
    rank
}

/// rank(Im Δ): rank of the image of Δ over all components and edges.
/// Rows are components; columns are edges.
pub fn rank_image_delta(d: &PrimaryEdgeField) -> usize {
    matrix_rank(&d.field, 1e-10)
}

/// rank(Im Σ): rank of the image of Σ over all components and edges.
/// Rows are components; columns are edges.
pub fn rank_image_sigma(s: &SigmaField) -> usize {
    matrix_rank(&s.field, 1e-10)
}

// ── ρ_P ──────────────────────────────────────────────────────────────────

/// Compute ρ_P classification.
///
/// C_X = propagation_capacity of the declared relational structure.
/// rank_sigma = rank(Im Σ).
///
/// Ceiling condition (G6):
///   If C_X = 0: Undefined{NoDeclaredComponents}
///   If rank_sigma >= C_X: Determined
///   If rank_sigma < C_X: CeilingBounded
///
/// Source: operators.rs V7, build plan V1.2 G6.
pub fn compute_rho_p(s: &SigmaField, rel: &DeclaredRelations) -> RhoP {
    let cx = rel.propagation_capacity();
    if cx == 0 {
        return RhoP::Undefined { reason: RhoPUndefinedReason::NoDeclaredComponents };
    }
    let rank_sigma = rank_image_sigma(s);
    if rank_sigma >= cx {
        RhoP::Determined { rank_sigma, cx }
    } else {
        RhoP::CeilingBounded { rank_sigma, cx }
    }
}

// ── Expression condition ──────────────────────────────────────────────────

/// expression_condition: true when rank(Im Σ) > 0 and C_X > 0.
/// Indicates the declared relational structure produces non-trivial
/// contrast accumulation under the Primary operators.
pub fn expression_condition(s: &SigmaField, rel: &DeclaredRelations) -> bool {
    let cx = rel.propagation_capacity();
    let rank_sigma = rank_image_sigma(s);
    cx > 0 && rank_sigma > 0
}

// ── Full Primary evaluation ───────────────────────────────────────────────

/// Complete Primary kernel evaluation on a single declared NodeField.
/// Returns all intermediate and final quantities.
/// E_primary = Σ(Δ(x)).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrimaryEvaluation {
    pub delta: PrimaryEdgeField,
    pub sigma: SigmaField,
    pub rank_delta: usize,
    pub rank_sigma: usize,
    pub cx: usize,
    pub rho_p: RhoP,
    pub expression_condition: bool,
}

pub fn evaluate_primary(x: &NodeField, rel: &DeclaredRelations) -> PrimaryEvaluation {
    let delta = operator_delta(x, rel);
    let sigma = operator_sigma(&delta, rel);
    let rank_delta = rank_image_delta(&delta);
    let rank_sigma = rank_image_sigma(&sigma);
    let cx = rel.propagation_capacity();
    let rho_p = compute_rho_p(&sigma, rel);
    let expr = expression_condition(&sigma, rel);
    PrimaryEvaluation {
        delta, sigma, rank_delta, rank_sigma, cx, rho_p, expression_condition: expr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::*;
    use crate::node_field::ObservationClass;

    fn si() -> ObservationClass { ObservationClass::SimulatedInput }

    #[test]
    fn delta_directed_difference() {
        // Δ(x)[e] = x[s] - x[t]
        let rel = fixture_f2_chain3(); // A→B→C
        let x = NodeField::single(vec![3.0, 1.0, 2.0], si());
        let d = operator_delta(&x, &rel);
        // edge 0: (0,1) → 3.0 - 1.0 = 2.0
        assert!((d.field[0][0] - 2.0).abs() < 1e-12, "Δ edge 0");
        // edge 1: (1,2) → 1.0 - 2.0 = -1.0
        assert!((d.field[0][1] - (-1.0)).abs() < 1e-12, "Δ edge 1");
    }

    #[test]
    fn sigma_no_successor_equals_delta() {
        // For terminal edges (adj_plus empty), Σ(d)[e] = d[e]
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![3.0, 1.0, 2.0], si());
        let d = operator_delta(&x, &rel);
        let s = operator_sigma(&d, &rel);
        // edge 1 is terminal (no adj_plus)
        assert!((s.field[0][1] - d.field[0][1]).abs() < 1e-12,
            "terminal edge: Σ must equal Δ");
    }

    #[test]
    fn sigma_with_successor() {
        // edge 0 has adj_plus = [edge 1]
        // Σ[0] = d[0] - d[1]
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![3.0, 1.0, 2.0], si());
        let d = operator_delta(&x, &rel);
        let s = operator_sigma(&d, &rel);
        let expected = d.field[0][0] - d.field[0][1];
        assert!((s.field[0][0] - expected).abs() < 1e-12,
            "Σ with successor: antisymmetric accumulation");
    }

    #[test]
    fn g6_rho_p_pair_undefined() {
        // F1 PRC-2: C_X=0 → Undefined
        let rel = fixture_f1_pair();
        let x = NodeField::single(vec![1.0, 2.0], si());
        let eval = evaluate_primary(&x, &rel);
        assert_eq!(eval.cx, 0);
        assert_eq!(eval.rho_p, RhoP::Undefined {
            reason: RhoPUndefinedReason::NoDeclaredComponents
        });
    }

    #[test]
    fn g6_rho_p_chain3_k1_determined() {
        // F2 chain3: C_X=1, k=1 → Determined
        let rel = fixture_f2_chain3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        let eval = evaluate_primary(&x, &rel);
        assert_eq!(eval.cx, 1);
        assert!(matches!(eval.rho_p, RhoP::Determined { .. }));
    }

    #[test]
    fn g6_rho_p_closed_relational_3_k1_ceiling_bounded() {
        // F3 3LCT: C_X=3, k=1 → CeilingBounded (rank_sigma ≤ 1 < 3)
        let rel = fixture_f3_closed_relational_3();
        let x = NodeField::single(vec![1.0, 2.0, 3.0], si());
        let eval = evaluate_primary(&x, &rel);
        assert_eq!(eval.cx, 3);
        assert!(matches!(eval.rho_p, RhoP::CeilingBounded { .. }),
            "3LCT k=1 must be CeilingBounded");
    }

    #[test]
    fn g6_rho_p_chain4_k1_ceiling_bounded() {
        // Chain-4: C_X=2, k=1 → CeilingBounded
        let rel = fixture_f4_chain4();
        let x = NodeField::single(vec![1.0, 2.0, 3.0, 4.0], si());
        let eval = evaluate_primary(&x, &rel);
        assert_eq!(eval.cx, 2);
        assert!(matches!(eval.rho_p, RhoP::CeilingBounded { .. }));
    }

    #[test]
    fn expression_condition_false_for_pair() {
        let rel = fixture_f1_pair();
        let x = NodeField::single(vec![1.0, 2.0], si());
        let eval = evaluate_primary(&x, &rel);
        // C_X=0 → expression_condition must be false
        assert!(!eval.expression_condition);
    }

    #[test]
    fn all_outputs_finite() {
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
            let x = NodeField::single((0..n).map(|i| (i + 1) as f64).collect(), si());
            let eval = evaluate_primary(&x, &rel);
            for c in 0..eval.delta.n_components {
                for &v in &eval.delta.field[c] {
                    assert!(v.is_finite(), "Δ output must be finite");
                }
                for &v in &eval.sigma.field[c] {
                    assert!(v.is_finite(), "Σ output must be finite");
                }
            }
        }
    }
}
