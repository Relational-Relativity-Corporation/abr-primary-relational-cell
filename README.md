# abr-primary-relational-cell

**Metatron Dynamics, Inc.**
Primary Relational Cell Simulator — V0.1
Convention build — WeAreDevelopers San Jose

## Kernel authority

- `operators.rs` V7
- `derived_invariants.rs` V4.1
- Build plan V1.2

Every formula in this simulator is independently implemented against the kernel. Kernel files are reference only — not code dependencies.

## What this simulator does

Evaluates the Primary kernel `E_primary = Σ(Δ(x))` over ten declared relational topology fixtures, producing:

- Δ (directed contrast) and Σ (accumulated relational contrast) at each declared edge
- rank(Im Δ), rank(Im Σ), propagation capacity C_X
- ρ_P classification (Determined / CeilingBounded / Undefined)
- expression_condition
- Admissibility findings (CirculationCancellation, if produced by evaluation)
- Matched intervention comparison: three-locus closed relational topology vs. open chain

## Terminology discipline

**Three-locus closed relational topology (3LCT)**: abstract directed relational structure `V={A,B,C}, R={A→B, B→C, C→A}`. Carries no spatial coordinates, no material substrate, no physical geometry.

**Physical geometry**: how a substrate is arranged in space. Selected only after measurement requirements are known.

**Measurement correspondence M**: establishes whether measurements of a physical device supply the loci, components, and directed directed relations required by the mathematical declaration.

These three levels are never conflated.

## What is absent

- B operator (absent through V0.9)
- ABR operators (absent through V0.9)
- Persistence history
- Path accumulation
- Physical fabrication claims

## Running

```
cargo test
cargo run --bin simulate
```

Results are written to `results/`.

## Build plan

V1.2 — Verifier PASS pending. See build plan document for gate criteria G1–G13.

## Open conditions

- OC-PRC-1: observable distinction between relational information cell and arbitrary declared structure
- OC-PRC-2: minimum topology for consistent expression_condition
- OC-PRC-3: non-adjacent locus intervention propagation
- OC-ρP-1, OC-ρP-2: carried from kernel
- Physical correspondence: open — no material implementation selected

---

*Metatron Dynamics, Inc. — relationalrelativity.dev*
