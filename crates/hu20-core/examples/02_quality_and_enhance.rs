//! Exemplo 02 (Intermediário): Comparativo de qualidade antes/depois do realce de Gabor e transcodificação WSQ/TIFF.
//!
//! Uso:
//!   cargo run -p hu20-core --example 02_quality_and_enhance

use hu20_core::{RawFrame, enhance_gabor, evaluate_quality};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 02. Realce de Gabor & Transcodificação Multi-Formato ===");

    let captured_path = PathBuf::from("target/examples/01_basic/capture.png");
    let source_path = if captured_path.exists() {
        captured_path
    } else {
        PathBuf::from("tests/fixtures/degraded_noisy.png")
    };
    let degraded = RawFrame::from_png(&source_path)?;
    let q_before = evaluate_quality(&degraded);

    let enhanced = enhance_gabor(&degraded);
    let q_after = evaluate_quality(&enhanced);

    println!("Comparativo de Qualidade ({}):", source_path.display());
    println!(
        "  Antes  -> score={:3}/100 | variância={:7.1} | saturação={:5.1}% | aceito={}",
        *q_before.score,
        q_before.mean_local_variance,
        q_before.saturated_ratio * 100.0,
        q_before.accepted
    );
    println!(
        "  Depois -> score={:3}/100 | variância={:7.1} | saturação={:5.1}% | aceito={}",
        *q_after.score,
        q_after.mean_local_variance,
        q_after.saturated_ratio * 100.0,
        q_after.accepted
    );

    let out_dir = PathBuf::from("target/examples/02_enhance");
    std::fs::create_dir_all(&out_dir)?;
    let enhanced_png = out_dir.join("enhanced.png");
    let wsq_path = out_dir.join("enhanced.wsq");
    let tiff_path = out_dir.join("enhanced.tiff");

    enhanced.write_png(&enhanced_png)?;
    enhanced.write_wsq(&wsq_path)?;
    enhanced.write_tiff(&tiff_path)?;

    let wsq_restored = RawFrame::from_file(&wsq_path)?;
    let q_wsq = evaluate_quality(&wsq_restored);

    println!("\nComparativo de Tamanhos e Integridade:");
    println!(
        "  RAW (L8) : {:6} bytes | sha256={}",
        enhanced.pixels().len(),
        &enhanced.sha256()[..16]
    );
    println!(
        "  PNG      : {:6} bytes",
        std::fs::metadata(&enhanced_png)?.len()
    );
    println!(
        "  TIFF     : {:6} bytes",
        std::fs::metadata(&tiff_path)?.len()
    );
    println!(
        "  WSQ      : {:6} bytes | qualidade pós-roundtrip={}/100",
        std::fs::metadata(&wsq_path)?.len(),
        *q_wsq.score
    );

    Ok(())
}
