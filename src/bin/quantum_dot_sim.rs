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
// This is a simulation experiment, not a claim that a physical
// device evolves through these states. Physical correspondence
// requires separately declared M_in — see Phase 0 V3.
//
// Two declared topologies:
//   Open chain:   Q_A → Q_B → Q_C       (C_X = 1)
//   Closed 3LCT:  Q_A → Q_B → Q_C → Q_A (C_X = 3)
//
// Experiment structure:
//   E1: Three single-occupation states [1,0,0], [0,1,0], [0,0,1]
//       run against both topologies
//   E2: Disturbance at Q_A — compare pre/post relational profiles
//       on both topologies
//   E3: Two-electron states from Ansaloni observation record
//       [1,1,0], [1,0,1], [0,1,1], [1,1,1]

use std::fs;
use chrono::Local;
use prc::topology::*;
use prc::node_field::{NodeField, ObservationClass};
use prc::admissibility::{run_experiment, ExperimentRecord};
use prc::kernel::{RhoP, FailureMode};

const E_UNIT: f64 = 1.602_176_634e-19; // C — operators.rs V7

fn si() -> ObservationClass { ObservationClass::SimulatedInput }

fn occupation(n_a: f64, n_b: f64, n_c: f64) -> NodeField {
    NodeField::single(vec![n_a, n_b, n_c], si())
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

fn print_result(label: &str, rec: &ExperimentRecord, x: &[f64]) {
    println!("  {}", label);
    println!("    n[v]: Q_A={:.0}  Q_B={:.0}  Q_C={:.0}  \
              (q: {:.3e}, {:.3e}, {:.3e} C)",
        x[0], x[1], x[2],
        x[0]*E_UNIT, x[1]*E_UNIT, x[2]*E_UNIT);
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
    let rho_base = 0.3; // mid-range of declared [0.1, 0.5]

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║  ABR Quantum-Dot Loci Simulation — V0.1.3                                  ║");
    println!("║  Metatron Dynamics, Inc.                                                    ║");
    println!("║  Three simulated quantum-dot-like loci: Q_A, Q_B, Q_C                      ║");
    println!("║  Observable: electron occupation n[v] ∈ {{0,1}}                              ║");
    println!("║  q[v] = n[v] · E_UNIT  (E_UNIT = 1.602176634e-19 C)                        ║");
    println!("║  SimulatedInput — not a physical device claim                               ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  rho_base = {:.1} (mid-range of declared [0.1, 0.5])", rho_base);
    println!();

    let chain  = fixture_f2_chain3();   // Q_A→Q_B→Q_C
    let closed = fixture_f3_closed_relational_3(); // Q_A→Q_B→Q_C→Q_A

    let mut all_records: Vec<serde_json::Value> = vec![];

    // ── EXPERIMENT 1: Single-occupation states ────────────────────────────
    println!("EXPERIMENT 1 — Single-occupation states");
    println!("  One electron declared at each locus in turn");
    sep();

    let single_states: Vec<(&str, [f64;3])> = vec![
        ("n=[1,0,0]  electron at Q_A", [1.0, 0.0, 0.0]),
        ("n=[0,1,0]  electron at Q_B", [0.0, 1.0, 0.0]),
        ("n=[0,0,1]  electron at Q_C", [0.0, 0.0, 1.0]),
    ];

    for (label, occ) in &single_states {
        let x = occupation(occ[0], occ[1], occ[2]);
        println!("  ── Open chain  Q_A→Q_B→Q_C  (C_X=1)");
        let r = run_experiment(&format!("chain_{}", &label[2..7]), &x, &chain, rho_base);
        print_result(label, &r, occ);
        all_records.push(serde_json::to_value(&r).unwrap());
        println!();
        println!("  ── Closed 3LCT  Q_A→Q_B→Q_C→Q_A  (C_X=3)");
        let r = run_experiment(&format!("closed_{}", &label[2..7]), &x, &closed, rho_base);
        print_result(label, &r, occ);
        all_records.push(serde_json::to_value(&r).unwrap());
        sep();
    }

    // ── EXPERIMENT 2: Disturbance at Q_A ─────────────────────────────────
    println!("EXPERIMENT 2 — Disturbance at Q_A");
    println!("  Baseline: n=[1,0,0]. Disturbance: +0.5 at Q_A (x_A → 1.5)");
    println!("  Compare relational profile before and after, open vs closed");
    sep();

    let base_occ = [1.0f64, 0.0, 0.0];
    let dist_occ = [1.5f64, 0.0, 0.0];

    let x_base = occupation(base_occ[0], base_occ[1], base_occ[2]);
    let x_dist = occupation(dist_occ[0], dist_occ[1], dist_occ[2]);

    for (topo_label, rel) in [
        ("Open chain  Q_A→Q_B→Q_C", &chain),
        ("Closed 3LCT Q_A→Q_B→Q_C→Q_A", &closed),
    ] {
        println!("  ── {}", topo_label);
        let r_base = run_experiment("E2_base", &x_base, rel, rho_base);
        let r_dist = run_experiment("E2_dist", &x_dist, rel, rho_base);
        print_result("before disturbance n=[1,0,0]", &r_base, &base_occ);
        println!();
        print_result("after  disturbance n=[1.5,0,0]", &r_dist, &dist_occ);
        println!();
        // Show Σ difference
        if let (Some(s_b), Some(s_d)) = (&r_base.sigma_values, &r_dist.sigma_values) {
            let diff: Vec<String> = s_b[0].iter().zip(s_d[0].iter())
                .map(|(b, d)| format!("{:.4}", d - b))
                .collect();
            println!("    ΔΣ (disturbance effect): {:?}", diff);
        }
        all_records.push(serde_json::to_value(&r_base).unwrap());
        all_records.push(serde_json::to_value(&r_dist).unwrap());
        sep();
    }

    // ── EXPERIMENT 3: Ansaloni observation record states ──────────────────
    println!("EXPERIMENT 3 — Two/three-electron states from Ansaloni et al. (2020)");
    println!("  Nature Comms 11, 6399. DOI: 10.1038/s41467-020-20280-3");
    println!("  Observed charge configurations: (1,1,0),(1,0,1),(0,1,1),(1,1,1)");
    println!("  ObservationClass = SimulatedInput");
    println!("  Note: configurations admitted as declared states; physical M_in");
    println!("  for actual device correspondence declared separately in Phase 0 V3");
    sep();

    let ansaloni_states: Vec<(&str, [f64;3])> = vec![
        ("n=[1,1,0]  Ansaloni Fig.2a  transverse double", [1.0, 1.0, 0.0]),
        ("n=[1,0,1]  Ansaloni Fig.2b  longitudinal double", [1.0, 0.0, 1.0]),
        ("n=[0,1,1]  Ansaloni Fig.2c  diagonal double", [0.0, 1.0, 1.0]),
        ("n=[1,1,1]  Ansaloni Fig.2d  triple dot", [1.0, 1.0, 1.0]),
    ];

    for (label, occ) in &ansaloni_states {
        let x = occupation(occ[0], occ[1], occ[2]);
        println!("  ── Open chain  Q_A→Q_B→Q_C");
        let r = run_experiment(&format!("A_chain_{:.3}", occ[0]+occ[1]*10.0+occ[2]*100.0),
                               &x, &chain, rho_base);
        print_result(label, &r, occ);
        all_records.push(serde_json::to_value(&r).unwrap());
        println!();
        println!("  ── Closed 3LCT Q_A→Q_B→Q_C→Q_A");
        let r = run_experiment(&format!("A_closed_{:.3}", occ[0]+occ[1]*10.0+occ[2]*100.0),
                               &x, &closed, rho_base);
        print_result(label, &r, occ);
        all_records.push(serde_json::to_value(&r).unwrap());
        sep();
    }

    // ── Shuttling sequence ────────────────────────────────────────────────
    println!("EXPERIMENT 3b — Ansaloni shuttling sequence (Fig.4d)");
    println!("  Measured sequence: (0,1,1)→(1,0,1)→(1,1,0)→(0,1,1)");
    println!("  Closed 3LCT topology — each state is a separate declared observation");
    sep();

    let shuttle: Vec<(&str, [f64;3])> = vec![
        ("step 1: n=[0,1,1]", [0.0, 1.0, 1.0]),
        ("step 2: n=[1,0,1]", [1.0, 0.0, 1.0]),
        ("step 3: n=[1,1,0]", [1.0, 1.0, 0.0]),
        ("step 4: n=[0,1,1]  (cycle complete)", [0.0, 1.0, 1.0]),
    ];

    for (label, occ) in &shuttle {
        let x = occupation(occ[0], occ[1], occ[2]);
        let r = run_experiment(&format!("shuttle_{}", &label[..6]), &x, &closed, rho_base);
        print_result(label, &r, occ);
        all_records.push(serde_json::to_value(&r).unwrap());
        println!();
    }
    sep();

    // ── Write run record ──────────────────────────────────────────────────
    let run_record = serde_json::json!({
        "experiment": "quantum_dot_loci_simulation",
        "simulator": "abr-primary-relational-cell V0.1.3",
        "timestamp": timestamp,
        "declaration": {
            "loci": ["Q_A", "Q_B", "Q_C"],
            "observable": "electron occupation n[v] in {0, 1}",
            "charge_relation": "q[v] = n[v] * E_UNIT",
            "E_UNIT_C": E_UNIT,
            "observation_class": "SimulatedInput",
            "physical_claim": "none — simulation experiment only",
            "rho_base": rho_base,
            "rho_base_range": [0.1, 0.5]
        },
        "source_record": {
            "paper": "Ansaloni et al. (2020)",
            "doi": "10.1038/s41467-020-20280-3",
            "journal": "Nature Communications 11, 6399",
            "note": "Experiment 3 states drawn from this record. \
                     Physical M_in for device correspondence declared \
                     separately in Phase 0 V3."
        },
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
    println!("   This is a simulation of declared electron occupation states.");
    println!("   Physical correspondence to actual quantum-dot devices requires");
    println!("   separately declared M_in — see Phase 0 V3 (PASS/FREEZE).");
    println!();
}
