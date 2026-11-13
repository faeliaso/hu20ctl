//! Exemplo 04 (Avançado com Comparativo): Matching 1:1, busca 1:N, liveness e laudo forense.
//!
//! Uso:
//!   cargo run -p hu20-core --example 04_verify_and_compare

use hu20_core::{
    RawFrame, analyze_liveness, annotate_frame, compare_frames, compare_frames_image,
    compare_probe_to_gallery, create_forensic_report, extract_minutiae, match_minutiae_pairs,
};
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 04. Comparativo Biométrico 1:1, 1:N & Relatório Forense ===");

    let captured_path = PathBuf::from("target/examples/01_basic/capture.png");
    let probe_path = if captured_path.exists() {
        captured_path
    } else {
        PathBuf::from("tests/fixtures/probe.png")
    };
    println!("Probe utilizada: {}", probe_path.display());

    let probe = RawFrame::from_png(&probe_path)?;
    let ref_match = if probe_path.ends_with("capture.png") {
        // Simula segunda amostra da mesma digital com pequeno deslocamento/realce
        hu20_core::enhance_gabor(&probe)
    } else {
        RawFrame::from_png("tests/fixtures/reference_match.png")?
    };
    let ref_mismatch = RawFrame::from_png("tests/fixtures/reference_mismatch.png")?;
    let degraded = RawFrame::from_png("tests/fixtures/degraded_noisy.png")?;

    let threshold = 40.0_f32;
    let pairs = [
        ("probe vs probe (idêntico)", &probe),
        ("probe vs reference_match (mesmo dedo)", &ref_match),
        ("probe vs reference_mismatch (dedo distinto)", &ref_mismatch),
        ("probe vs degraded_noisy (ruído)", &degraded),
    ];

    println!("\n1) Comparativo 1:1 (threshold = {threshold:.1}):");
    for (label, candidate) in pairs {
        let report = compare_frames(&probe, candidate);
        let matched = *report.score >= threshold;
        println!(
            "  {:<44} -> score={:5.1}/100 | matched={:<5} | correspondências={}",
            label, *report.score, matched, report.matched_minutiae
        );
    }

    println!("\n2) Busca 1:N em Galeria:");
    let gallery = vec![
        ("user_mismatch".to_string(), ref_mismatch.clone()),
        ("user_degraded".to_string(), degraded.clone()),
        ("user_match".to_string(), ref_match.clone()),
    ];
    let gallery_report = compare_probe_to_gallery(&probe, &gallery, threshold);
    for (rank, entry) in gallery_report.matches.iter().enumerate() {
        println!(
            "  #{:<2} {:<15} -> score={:5.1}/100 | matched={}",
            rank + 1,
            entry.reference_path,
            *entry.score,
            entry.matched
        );
    }

    println!("\n3) Análise de Vivacidade (heurística estática) & Laudo Forense:");
    let liveness = analyze_liveness(&probe);
    println!(
        "  Liveness probe.png: score={}/100 | {:?} | band={:.2} | coerência={:.2}",
        liveness.liveness_score,
        liveness.classification,
        liveness.fft_ridge_band_energy,
        liveness.directional_coherence
    );

    let out_dir = PathBuf::from("target/examples/04_forensic");
    std::fs::create_dir_all(&out_dir)?;

    let probe_minutiae = extract_minutiae(&probe);
    let ref_minutiae = extract_minutiae(&ref_match);
    let pairs_matched = match_minutiae_pairs(&probe_minutiae, &ref_minutiae);

    let annotated_path = out_dir.join("probe_annotated.png");
    let comparison_path = out_dir.join("probe_vs_match.png");
    let report_path = out_dir.join("forensic_report.json");

    annotate_frame(&probe, &probe_minutiae).save(&annotated_path)?;
    compare_frames_image(
        &probe,
        &ref_match,
        &probe_minutiae,
        &ref_minutiae,
        &pairs_matched,
    )
    .save(&comparison_path)?;

    let report = create_forensic_report(Path::new(&probe_path), &probe);
    std::fs::write(&report_path, serde_json::to_string_pretty(&report)?)?;

    println!("Artefatos forenses gerados em: {}", out_dir.display());
    Ok(())
}
