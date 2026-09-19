// quantum_dot_sim.rs — Metatron Dynamics, Inc.
// abr-primary-relational-cell V0.1.3
//
// Simulation experiment: electron-associated occupation states
// at three quantum-dot-like loci Q_A, Q_B, Q_C.
//
// Origin declaration:
//   Three simulated quantum-dot-like loci: Q_A, Q_B, Q_C
//   Observable: electron occupation n[v] ∈ {0, 1}
//   Declared charge relation: q[v] = n[v] · E_UNIT
//   E_UNIT = 1.602_176_634e-19 C (operators.rs V7)
//   ObservationClass = SimulatedInput
//
// Experiment 2 uses a separate declared scalar field x → x'
// (SimulatedInput, continuous). x'=1.5 is NOT electron occupation
// and is NOT converted to charge via E_UNIT. It is a declared
// simulated perturbation of an arbitrary scalar field.
//
// This is a simulation experiment, not a claim that a physical
// device evolves through these states. Physical correspondence
// requires separately declared M_in — see Phase 0 V3.

use std::fs;
use chrono::Local;
use prc::topology::*;
use prc::node_field::{NodeField, ObservationClass};
use prc::admissibility::{run_experiment, ExperimentRecord};
use prc::kernel::{RhoP, FailureMode};

const E_UNIT: f64 = 1.602_176_634e-19; // C — operators.rs V7

fn si() -> ObservationClass { ObservationClass::SimulatedInput }

/// Construct NodeField from integer electron occupation values.
/// n[v] ∈ {0, 1} per Origin declaration.
fn occupation(n_a: u8, n_b: u8, n_c: u8) -> NodeField {
    assert!(n_a <= 1 && n_b <= 1 && n_c <= 1,
        "n[v] must be in {{0,1}} per Origin declaration");
    NodeField::single(vec![n_a as f64, n_b as f64, n_c as f64], si())
}

/// Construct NodeField from arbitrary continuous scalar values.
/// Used only in Experiment 2 — NOT electron occupation.
/// E_UNIT is NOT applied to these values.
fn scalar_field(x_a: f64, x_b: f64, x_c: f64) -> NodeField {
    NodeField::single(vec![x_a, x_b, x_c], si())
}

fn rho_p_str(rho: &Option<RhoP>) -> String {
    match rho {
        None => "—".to_string(),
        Some(RhoP::Determined { value, .. }) =>
            format!("Determined  ρ_P={:.4}", value),
        Some(RhoP::CeilingBounded { value, ceiling, .. }) =>
            format!("CeilingBounded  ρ_P={:.4}  ceil={:.4}", value, ceiling),
        Some(RhoP::Undefined { .. }) => "Undefined".to_string(),
    }
}

fn status_str(rec: &ExperimentRecord) -> &'static str {
    match &rec.failure_mode {
        Some(FailureMode::DifferentiationCollapse) => "FM1 DifferentiationCollapse",
        Some(FailureMode::RelationalIsolation)     => "FM2 RelationalIsolation",
        None => match &rec.admissibility_finding {
            Some(_) => "CirculationCancellation",
            None    => "Admissible",
        },
    }
}

fn sep() { println!("{}", "─".repeat(80)); }

/// Print result for an electron occupation NodeField.
/// n values are integers; charge is displayed via E_UNIT.
fn print_occupation_result(label: &str, rec: &ExperimentRecord, n: [u8;3]) {
    println!("  {}", label);
    println!("    n[v]: Q_A={}  Q_B={}  Q_C={}  \
              (q: {:.3e}, {:.3e}, {:.3e} C)",
        n[0], n[1], n[2],
        n[0] as f64 * E_UNIT,
        n[1] as f64 * E_UNIT,
        n[2] as f64 * E_UNIT);
    print_kernel_output(rec);
}

/// Print result for a continuous scalar field NodeField.
/// No occupation label, no E_UNIT conversion.
fn print_scalar_result(label: &str, rec: &ExperimentRecord, x: [f64;3]) {
    println!("  {}", label);
    println!("    x[v]: Q_A={:.4}  Q_B={:.4}  Q_C={:.4}  \
              (continuous scalar field — not electron occupation)",
        x[0], x[1], x[2]);
    print_kernel_output(rec);
}

fn print_kernel_output(rec: &ExperimentRecord) {
    if let Some(d) = &rec.delta_values {
        println!("    Δ(x): {:?}", d[0].iter()
            .map(|v| format!("{:.4}", v)).collect::<Vec<_>>());
    }
    if let Some(s) = &rec.sigma_values {
        println!("    Σ(Δ): {:?}", s[0].iter()
            .map(|v| format!("{:.4}", v)).collect::<Vec<_>>());
    }
    if let Some(a) = &rec.antisym_values {
        println!("    antisym: {:?}", a[0].iter()
            .map(|v| format!("{:.4}", v)).collect::<Vec<_>>());
    }
    println!("    rank_Δ={}  rank_Σ={}  C_X={}  {}  → {}",
        rec.rank_delta.unwrap_or(0),
        rec.rank_sigma.unwrap_or(0),
        rec.cx,
        rho_p_str(&rec.rho_p),
        status_str(rec));
}

fn main() {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let rho_base = 0.3;

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║  ABR Quantum-Dot Loci Simulation — V0.1.3                                  ║");
    println!("║  Metatron Dynamics, Inc.                                                    ║");
    println!("║  Three simulated quantum-dot-like loci: Q_A, Q_B, Q_C                      ║");
    println!("║  Observable: electron occupation n[v] ∈ {{0,1}}                              ║");
    println!("║  q[v] = n[v] · E_UNIT  (E_UNIT = 1.602176634e-19 C)                        ║");
    println!("║  Experiment 2: separate continuous scalar field — not occupation             ║");
    println!("║  SimulatedInput — not a physical device claim                               ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  rho_base = {:.1} (mid-range of declared [0.1, 0.5])", rho_base);
    println!();

    let chain  = fixture_f2_chain3();
    let closed = fixture_f3_closed_relational_3();

    let mut all_records: Vec<serde_json::Value> = vec![];

    // ── EXPERIMENT 1: Single-occupation states ────────────────────────────
    println!("EXPERIMENT 1 — Single-occupation states  n[v] ∈ {{0,1}}");
    println!("  One electron declared at each locus in turn");
    sep();

    let single: Vec<(&str, [u8;3])> = vec![
        ("n=[1,0,0]  electron at Q_A", [1,0,0]),
        ("n=[0,1,0]  electron at Q_B", [0,1,0]),
        ("n=[0,0,1]  electron at Q_C", [0,0,1]),
    ];

    for (label, n) in &single {
        let x = occupation(n[0], n[1], n[2]);
        println!("  ── Open chain  Q_A→Q_B→Q_C  (C_X=1)");
        let r = run_experiment(&format!("E1_chain_{}{}{}", n[0],n[1],n[2]),
                               &x, &chain, rho_base);
        print_occupation_result(label, &r, *n);
        all_records.push(serde_json::to_value(&r).unwrap());
        println!();
        println!("  ── Closed 3LCT  Q_A→Q_B→Q_C→Q_A  (C_X=3)");
        let r = run_experiment(&format!("E1_closed_{}{}{}", n[0],n[1],n[2]),
                               &x, &closed, rho_base);
        print_occupation_result(label, &r, *n);
        all_records.push(serde_json::to_value(&r).unwrap());
        sep();
    }

    // ── EXPERIMENT 2: Continuous scalar perturbation ──────────────────────
    println!("EXPERIMENT 2 — Continuous scalar field perturbation");
    println!("  x and x' are declared scalar fields — NOT electron occupation");
    println!("  E_UNIT is NOT applied. No charge interpretation.");
    println!("  Purpose: test relational response to continuous perturbation");
    println!("  at Q_A across open and closed topology.");
    sep();

    let x_base = [1.0f64, 0.0, 0.0];
    let x_dist = [1.5f64, 0.0, 0.0];

    for (topo_label, rel) in [
        ("Open chain  Q_A→Q_B→Q_C", &chain),
        ("Closed 3LCT Q_A→Q_B→Q_C→Q_A", &closed),
    ] {
        println!("  ── {}", topo_label);
        let xb = scalar_field(x_base[0], x_base[1], x_base[2]);
        let xd = scalar_field(x_dist[0], x_dist[1], x_dist[2]);
        let r_base = run_experiment("E2_base", &xb, rel, rho_base);
        let r_dist = run_experiment("E2_dist", &xd, rel, rho_base);
        print_scalar_result("x  = [1.0, 0.0, 0.0]  (baseline)", &r_base, x_base);
        println!();
        print_scalar_result("x' = [1.5, 0.0, 0.0]  (perturbation)", &r_dist, x_dist);
        println!();
        if let (Some(s_b), Some(s_d)) = (&r_base.sigma_values, &r_dist.sigma_values) {
            let diff: Vec<String> = s_b[0].iter().zip(s_d[0].iter())
                .map(|(b, d)| format!("{:.4}", d - b))
                .collect();
            println!("    ΔΣ (perturbation effect on relational contrast): {:?}", diff);
        }
        all_records.push(serde_json::to_value(&r_base).unwrap());
        all_records.push(serde_json::to_value(&r_dist).unwrap());
        sep();
    }

    // ── EXPERIMENT 3: Ansaloni observation record states ──────────────────
    println!("EXPERIMENT 3 — Two/three-electron states from Ansaloni et al. (2020)");
    println!("  Nature Comms 11, 6399. DOI: 10.1038/s41467-020-20280-3");
    println!("  Observed charge configurations: (1,1,0),(1,0,1),(0,1,1),(1,1,1)");
    println!("  Admitted as declared occupation states n[v] ∈ {{0,1}}");
    println!("  Physical M_in for device correspondence declared separately (Phase 0 V3)");
    sep();

    let ansaloni: Vec<(&str, [u8;3])> = vec![
        ("n=[1,1,0]  Ansaloni Fig.2a  transverse double", [1,1,0]),
        ("n=[1,0,1]  Ansaloni Fig.2b  longitudinal double", [1,0,1]),
        ("n=[0,1,1]  Ansaloni Fig.2c  diagonal double", [0,1,1]),
        ("n=[1,1,1]  Ansaloni Fig.2d  triple dot", [1,1,1]),
    ];

    for (label, n) in &ansaloni {
        let x = occupation(n[0], n[1], n[2]);
        println!("  ── Open chain  Q_A→Q_B→Q_C");
        let r = run_experiment(&format!("E3_chain_{}{}{}", n[0],n[1],n[2]),
                               &x, &chain, rho_base);
        print_occupation_result(label, &r, *n);
        all_records.push(serde_json::to_value(&r).unwrap());
        println!();
        println!("  ── Closed 3LCT Q_A→Q_B→Q_C→Q_A");
        let r = run_experiment(&format!("E3_closed_{}{}{}", n[0],n[1],n[2]),
                               &x, &closed, rho_base);
        print_occupation_result(label, &r, *n);
        all_records.push(serde_json::to_value(&r).unwrap());
        sep();
    }

    // ── EXPERIMENT 3b: Shuttling sequence ────────────────────────────────
    println!("EXPERIMENT 3b — Ansaloni shuttling sequence (Fig.4d)");
    println!("  Measured sequence: (0,1,1)→(1,0,1)→(1,1,0)→(0,1,1)");
    println!("  Closed 3LCT — each state is a separate declared observation");
    println!("  Σ(Δ(x_r)) does not generate x_{{r+1}} — G8 preserved");
    sep();

    let shuttle: Vec<(&str, [u8;3])> = vec![
        ("step 1: n=[0,1,1]", [0,1,1]),
        ("step 2: n=[1,0,1]", [1,0,1]),
        ("step 3: n=[1,1,0]", [1,1,0]),
        ("step 4: n=[0,1,1]  (cycle complete)", [0,1,1]),
    ];

    for (label, n) in &shuttle {
        let x = occupation(n[0], n[1], n[2]);
        let r = run_experiment(&format!("E3b_{}", &label[..6]),
                               &x, &closed, rho_base);
        print_occupation_result(label, &r, *n);
        all_records.push(serde_json::to_value(&r).unwrap());
        println!();
    }

    // Bounded finding statement
    println!("  Bounded finding from this run:");
    println!("  Cyclic permutation of the declared occupation configuration");
    println!("  corresponds to cyclic permutation of the Σ and antisymmetric");
    println!("  profiles on the closed 3LCT topology. Return of the declared");
    println!("  occupation configuration is accompanied by return of the same");
    println!("  relational output. No causal claim is made.");
    sep();

    // ── Write run record ──────────────────────────────────────────────────
    let run_record = serde_json::json!({
        "experiment": "quantum_dot_loci_simulation",
        "version": "V0.1.3-corrected",
        "simulator": "abr-primary-relational-cell V0.1.3",
        "timestamp": timestamp,
        "declaration": {
            "loci": ["Q_A", "Q_B", "Q_C"],
            "occupation_observable": "n[v] in {0, 1}",
            "charge_relation": "q[v] = n[v] * E_UNIT",
            "E_UNIT_C": E_UNIT,
            "experiment_2_field": "continuous scalar SimulatedInput — not occupation",
            "observation_class": "SimulatedInput",
            "physical_claim": "none — simulation experiment only",
            "rho_base": rho_base,
            "rho_base_range": [0.1, 0.5]
        },
        "source_record": {
            "paper": "Ansaloni et al. (2020)",
            "doi": "10.1038/s41467-020-20280-3",
            "journal": "Nature Communications 11, 6399",
            "note": "Experiment 3/3b states from this record. \
                     Physical M_in declared separately in Phase 0 V3."
        },
        "bounded_finding_3b": "Cyclic permutation of declared occupation \
            corresponds to cyclic permutation of Sigma and antisymmetric \
            profiles on the closed 3LCT. No causal claim made.",
        "records": all_records,
    });

    fs::create_dir_all("results").expect("could not create results/");
    let outpath = format!("results/qdot_sim_{}.json", timestamp);
    fs::write(&outpath, serde_json::to_string_pretty(&run_record).unwrap())
        .expect("could not write run record");

    println!();
    println!("Run record written: {}", outpath);
    println!();
    println!("── Claim boundary ──────────────────────────────────────────────");
    println!("   Simulation of declared electron occupation states.");
    println!("   Physical correspondence requires separately declared M_in");
    println!("   — see Phase 0 V3 (PASS/FREEZE).");
    println!();
}
