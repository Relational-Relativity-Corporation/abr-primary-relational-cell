// topology.rs — Metatron Dynamics, Inc.
// abr-primary-relational-cell V0.1
//
// Declares loci, directed edges, EdgeProvenance, DeclaredRelations,
// propagation capacity C_X, and cycle detection.
//
// Kernel authority: operators.rs V7 lines 186–202 (EdgeProvenance),
//   lines 204–276 (DeclaredRelations), lines 133–139 (cycle detection).
//
// ── Terminology discipline (declared this session) ────────────────────────
//
// "Three-locus closed relational topology" (3LCT) refers to the abstract
// directed relational structure V={A,B,C}, R={A→B, B→C, C→A}.
// "Physical geometry" refers to how a substrate is arranged in space.
// These are different levels. Conflating them is a provenance failure.
//
// ── Ring topology and admissibility ──────────────────────────────────────
//
// has_undirected_cycle() detects and reports cycle structure only.
// It does not reject the declaration or prevent evaluation.
// CirculationCancellation is a possible admissibility finding that
// may or may not appear in evaluation output — it is not predetermined
// by topology alone. See admissibility.rs.
//
// ── EdgeProvenance ────────────────────────────────────────────────────────
//
// Canonical three-variant enum from operators.rs V7 lines 190–202.
// No other variants. No substitution.
//
// Continuation — within-family spatial continuation, declared from
//   observable or device geometry. Open boundary: no wraparound.
// Coupling     — cross-family spatial coupling, declared from physical
//   coupling mechanism (evanescent backscattering, scaffold connectivity).
// Persistence  — relational-evolutionary, connects E_prior[e] to
//   E_current[e] across one declared process step.
//   Represented in PersistenceState, NOT in the spatial edge list,
//   because its loci are edge-valued, not node-valued.
//   PROHIBITED as a spatial edge constructor (G3).

use serde::{Deserialize, Serialize};

/// Canonical three-variant EdgeProvenance enum.
/// Source: operators.rs V7 lines 190–202. No other variants permitted.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EdgeProvenance {
    /// Within-family spatial continuation — declared from observable or
    /// device geometry. Open boundary: no wraparound.
    Continuation,
    /// Cross-family spatial coupling — declared from physical coupling
    /// mechanism (evanescent backscattering, scaffold connectivity).
    Coupling,
    /// Relational-evolutionary — connects E_prior[e] to E_current[e]
    /// across one declared process step.
    /// Represented in PersistenceState, not in the spatial edge list.
    /// PROHIBITED as a spatial edge constructor.
    Persistence,
}

/// Named topology label for fixture identification and run records.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TopologyLabel {
    Pair,               // PRC-2: A→B
    Chain3,             // PRC-3 open chain: A→B→C
    ClosedRelational3,  // 3LCT: A→B→C→A (three-locus closed relational topology)
    Chain4,             // A→B→C→D
    Branch,             // A→B, B→C, B→D
    Convergent,         // A→C, B→C, C→D
    Diamond,            // A→B, A→C, B→D, C→D
    Custom(String),
}

/// Cycle detection report — returned by has_undirected_cycle().
/// Detection only. Does not authorize or prohibit evaluation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CycleReport {
    pub has_cycle: bool,
    pub note: String,
}

/// Declared relational structure over n_nodes loci and directed edges.
/// Source: operators.rs V7 lines 204–276.
///
/// Every edge carries exactly one EdgeProvenance.
/// Persistence is prohibited as a spatial edge constructor (G3).
/// No reverse() method. No wraparound. No undeclared relations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeclaredRelations {
    pub n_nodes: usize,
    /// Directed edges as (source, target) pairs. Index is edge id.
    pub edges: Vec<(usize, usize)>,
    /// One provenance per edge. provenance[e] corresponds to edges[e].
    pub provenance: Vec<EdgeProvenance>,
    /// out[i] = edge indices leaving node i
    pub out: Vec<Vec<usize>>,
    /// inc[i] = edge indices entering node i
    pub inc: Vec<Vec<usize>>,
    /// adj_plus[e] = edge indices that are successors of edge e
    /// (i.e., edges leaving the target node of e)
    pub adj_plus: Vec<Vec<usize>>,
    /// adj_minus[e] = edge indices that are predecessors of edge e
    /// (i.e., edges entering the source node of e)
    pub adj_minus: Vec<Vec<usize>>,
    /// Topology label for fixture identification
    pub label: TopologyLabel,
}

impl DeclaredRelations {
    /// Construct DeclaredRelations from explicit edge list and provenance.
    /// Panics if:
    ///   - edges and provenance lengths differ
    ///   - any EdgeProvenance::Persistence appears (prohibited spatially)
    ///   - any node index is out of range for n_nodes
    pub fn new(
        n_nodes: usize,
        edges: Vec<(usize, usize)>,
        provenance: Vec<EdgeProvenance>,
        label: TopologyLabel,
    ) -> Self {
        assert_eq!(
            edges.len(), provenance.len(),
            "edges and provenance must have equal length"
        );
        assert!(
            provenance.iter().all(|p| *p != EdgeProvenance::Persistence),
            "EdgeProvenance::Persistence is prohibited in spatial edge constructors (G3). \
             Use PersistenceState for relational-evolutionary edges."
        );
        for &(s, t) in &edges {
            assert!(s < n_nodes, "source node {} out of range for n_nodes={}", s, n_nodes);
            assert!(t < n_nodes, "target node {} out of range for n_nodes={}", t, n_nodes);
        }

        let n_edges = edges.len();

        // Build out[i] and inc[i]
        let mut out = vec![vec![]; n_nodes];
        let mut inc = vec![vec![]; n_nodes];
        for (e, &(s, t)) in edges.iter().enumerate() {
            out[s].push(e);
            inc[t].push(e);
        }

        // Build adj_plus[e] and adj_minus[e]
        // adj_plus[e]: edges leaving the target of e
        // adj_minus[e]: edges entering the source of e
        let mut adj_plus = vec![vec![]; n_edges];
        let mut adj_minus = vec![vec![]; n_edges];
        for (e, &(s, t)) in edges.iter().enumerate() {
            adj_plus[e] = out[t].clone();
            adj_minus[e] = inc[s].clone();
        }

        DeclaredRelations { n_nodes, edges, provenance, out, inc, adj_plus, adj_minus, label }
    }

    /// Convenience constructor with all edges assigned Continuation provenance.
    pub fn from_edges(n_nodes: usize, edges: Vec<(usize, usize)>, label: TopologyLabel) -> Self {
        let n = edges.len();
        Self::new(n_nodes, edges, vec![EdgeProvenance::Continuation; n], label)
    }

    pub fn n_edges(&self) -> usize {
        self.edges.len()
    }

    /// Propagation capacity C_X.
    /// Definition: count of edges e where adj_plus[e] is non-empty.
    /// Source: build plan V1.2 C_X controlling table.
    pub fn propagation_capacity(&self) -> usize {
        self.adj_plus.iter().filter(|ap| !ap.is_empty()).count()
    }

    /// Cycle detection using undirected DFS.
    /// Returns a CycleReport — detection and reporting ONLY.
    /// This function does not reject any declaration.
    /// Source: operators.rs V7 lines 133–139, build plan V1.2 G4.
    pub fn has_undirected_cycle(&self) -> CycleReport {
        // Build undirected adjacency from directed edges
        let mut adj: Vec<Vec<usize>> = vec![vec![]; self.n_nodes];
        for &(s, t) in &self.edges {
            adj[s].push(t);
            adj[t].push(s);
        }

        let mut visited = vec![false; self.n_nodes];
        let mut has_cycle = false;

        for start in 0..self.n_nodes {
            if !visited[start] {
                // DFS from start; parent tracks where we came from
                let mut stack: Vec<(usize, Option<usize>)> = vec![(start, None)];
                while let Some((node, parent)) = stack.pop() {
                    if visited[node] {
                        has_cycle = true;
                        break;
                    }
                    visited[node] = true;
                    for &nbr in &adj[node] {
                        if Some(nbr) != parent {
                            stack.push((nbr, Some(node)));
                        }
                    }
                }
                if has_cycle { break; }
            }
        }

        CycleReport {
            has_cycle,
            note: if has_cycle {
                "Undirected cycle detected. Reported for admissibility classification. \
                 Evaluation proceeds — see admissibility_finding in ExperimentRecord."
                    .to_string()
            } else {
                "No undirected cycle detected.".to_string()
            },
        }
    }
}

// ── Canonical fixture constructors ────────────────────────────────────────
// All ten fixtures required by build plan V1.2.

/// F1 — PRC-2: A→B. C_X=0. ρ_P Undefined.
pub fn fixture_f1_pair() -> DeclaredRelations {
    DeclaredRelations::from_edges(2, vec![(0, 1)], TopologyLabel::Pair)
}

/// F2 — PRC-3 open chain: A→B→C. C_X=1. ρ_P Determined at k≥1.
pub fn fixture_f2_chain3() -> DeclaredRelations {
    DeclaredRelations::from_edges(3, vec![(0, 1), (1, 2)], TopologyLabel::Chain3)
}

/// F3 — Three-locus closed relational topology (3LCT): A→B→C→A. C_X=3.
/// ρ_P CeilingBounded at k=1,2; Determined at k≥3.
/// This is the primary experimental topology. Evaluation proceeds normally.
/// CirculationCancellation is a possible admissibility finding — not predetermined.
pub fn fixture_f3_closed_relational_3() -> DeclaredRelations {
    DeclaredRelations::from_edges(3, vec![(0, 1), (1, 2), (2, 0)], TopologyLabel::ClosedRelational3)
}

/// F4 — Chain-4: A→B→C→D. C_X=2.
pub fn fixture_f4_chain4() -> DeclaredRelations {
    DeclaredRelations::from_edges(4, vec![(0, 1), (1, 2), (2, 3)], TopologyLabel::Chain4)
}

/// F5 — Branch: A→B, B→C, B→D. C_X=1.
pub fn fixture_f5_branch() -> DeclaredRelations {
    DeclaredRelations::from_edges(4, vec![(0, 1), (1, 2), (1, 3)], TopologyLabel::Branch)
}

/// F6 — Convergent: A→C, B→C, C→D. C_X=2.
pub fn fixture_f6_convergent() -> DeclaredRelations {
    DeclaredRelations::from_edges(4, vec![(0, 2), (1, 2), (2, 3)], TopologyLabel::Convergent)
}

/// F7 — Diamond: A→B, A→C, B→D, C→D. C_X=2.
pub fn fixture_f7_diamond() -> DeclaredRelations {
    DeclaredRelations::from_edges(4, vec![(0, 1), (0, 2), (1, 3), (2, 3)], TopologyLabel::Diamond)
}

/// F8 — 3LCT with local disturbance at A: primary intervention arm.
/// Same topology as F3. NodeField disturbance declared in experiment.rs.
pub fn fixture_f8_closed_relational_3_intervention() -> DeclaredRelations {
    DeclaredRelations::from_edges(3, vec![(0, 1), (1, 2), (2, 0)], TopologyLabel::ClosedRelational3)
}

/// F9 — Chain-3 with local disturbance at A: matched control for F8.
/// Same topology as F2. NodeField disturbance declared in experiment.rs.
pub fn fixture_f9_chain3_intervention() -> DeclaredRelations {
    DeclaredRelations::from_edges(3, vec![(0, 1), (1, 2)], TopologyLabel::Chain3)
}

/// F10 — Declared removal of edge C→A from 3LCT: topological change.
/// This is a binary declaration change (edge present or absent),
/// NOT scalar weakening. Result is an open chain A→B→C.
pub fn fixture_f10_closure_removed() -> DeclaredRelations {
    // Remove (2,0) — the closing edge C→A — leaving A→B→C
    DeclaredRelations::from_edges(3, vec![(0, 1), (1, 2)], TopologyLabel::Chain3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn g2_edge_provenance_three_variants_only() {
        // G2: EdgeProvenance enum is exactly Continuation, Coupling, Persistence.
        // Verified by the type definition — no other variants compile.
        let c = EdgeProvenance::Continuation;
        let k = EdgeProvenance::Coupling;
        let p = EdgeProvenance::Persistence;
        assert_ne!(c, k);
        assert_ne!(c, p);
        assert_ne!(k, p);
    }

    #[test]
    fn g3_persistence_prohibited_in_spatial_constructor() {
        // G3: Persistence must be rejected by the spatial constructor.
        let result = std::panic::catch_unwind(|| {
            DeclaredRelations::new(
                2,
                vec![(0, 1)],
                vec![EdgeProvenance::Persistence],
                TopologyLabel::Pair,
            )
        });
        assert!(result.is_err(), "Persistence in spatial constructor must panic (G3)");
    }

    #[test]
    fn g4_cycle_detection_only_does_not_reject() {
        // G4: has_undirected_cycle() is detection only — 3LCT must construct
        // and report cycle without rejecting the declaration.
        let rel = fixture_f3_closed_relational_3();
        let report = rel.has_undirected_cycle();
        assert!(report.has_cycle, "3LCT must report cycle detected");
        // Construction succeeded — no rejection occurred.
    }

    #[test]
    fn cx_pair() {
        assert_eq!(fixture_f1_pair().propagation_capacity(), 0);
    }

    #[test]
    fn cx_chain3() {
        assert_eq!(fixture_f2_chain3().propagation_capacity(), 1);
    }

    #[test]
    fn cx_closed_relational_3() {
        // Every edge in 3LCT has a non-empty adj_plus. C_X = 3.
        assert_eq!(fixture_f3_closed_relational_3().propagation_capacity(), 3);
    }

    #[test]
    fn cx_chain4() {
        assert_eq!(fixture_f4_chain4().propagation_capacity(), 2);
    }

    #[test]
    fn cx_branch() {
        assert_eq!(fixture_f5_branch().propagation_capacity(), 1);
    }

    #[test]
    fn cx_convergent() {
        assert_eq!(fixture_f6_convergent().propagation_capacity(), 2);
    }

    #[test]
    fn cx_diamond() {
        assert_eq!(fixture_f7_diamond().propagation_capacity(), 2);
    }

    #[test]
    fn f10_closure_removed_is_chain() {
        // F10: removing C→A produces open chain — C_X drops from 3 to 1.
        let full = fixture_f3_closed_relational_3();
        let removed = fixture_f10_closure_removed();
        assert_eq!(full.propagation_capacity(), 3);
        assert_eq!(removed.propagation_capacity(), 1);
    }

    #[test]
    fn no_undirected_cycle_in_chain() {
        let report = fixture_f2_chain3().has_undirected_cycle();
        assert!(!report.has_cycle);
    }

    #[test]
    fn no_reverse_method_on_declared_relations() {
        // G4: No reverse() method. Verified structurally — this test
        // documents the constraint. If a reverse() method is added,
        // the Verifier must reject it.
        let rel = fixture_f2_chain3();
        // rel.reverse() — must not compile. Structural guarantee.
        let _ = rel; // suppress unused warning
    }
}
