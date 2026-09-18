// lib.rs — Metatron Dynamics, Inc.
// abr-primary-relational-cell V0.1
// Convention build — WeAreDevelopers San Jose
//
// Kernel authority: operators.rs V7, derived_invariants.rs V4.1
// Build plan authority: ABR Primary Relational Cell Simulator V1.2
//
// Every formula in this simulator is independently implemented against
// the kernel. Kernel files are reference only — not a code dependency.
//
// B absent throughout. ABR operators absent through V0.9.
// No persistence history. No path accumulation.

pub mod topology;
pub mod node_field;
pub mod kernel;
pub mod admissibility;
pub mod experiment;
