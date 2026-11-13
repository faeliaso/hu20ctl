//! Exemplo 06 (Avançado - Custódia & 8 Formatos): Cadeia de custódia (.meta.json) e comparativo de todos os codecs.
//!
//! Uso:
//!   cargo run -p hu20-core --example 06_custody_and_formats

use hu20_core::{
    CaptureManifest, DPI, DerivedManifest, FRAME_LEN, HEIGHT, ImageFormat, Manifest, RawFrame,
    WIDTH, evaluate_quality, sha256_bytes,
};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 06. Cadeia de Custódia & Comparativo dos 8 Formatos ===");

    let captured_path = PathBuf::from("target/examples/01_basic/capture.png");
    let source_path = if captured_path.exists() {
        captured_path
    } else {
        PathBuf::from("tests/fixtures/probe.png")
    };
    println!("Origem: {}", source_path.display());

    let frame = RawFrame::from_file(&source_path)?;
    let raw_hash = frame.sha256();
    let quality = evaluate_quality(&frame);

    let out_dir = PathBuf::from("target/examples/06_custody");
    std::fs::create_dir_all(&out_dir)?;

    let master_png = out_dir.join("master.png");
    frame.write_png(&master_png)?;

    let capture_manifest = CaptureManifest {
        schema_version: 1,
        captured_at_utc: chrono::Utc::now(),
        sha256_raw_frame: raw_hash.clone(),
        raw_frame_bytes: FRAME_LEN,
        width: WIDTH,
        height: HEIGHT,
        dpi: DPI,
        quality,
        tool_version: env!("CARGO_PKG_VERSION").into(),
        driver_abi_version: None,
        device: None,
        standard_afis: true,
        format: ImageFormat::Png,
    };
    capture_manifest.write_adjacent(&master_png)?;

    let formats = [
        ("png", ImageFormat::Png),
        ("tiff", ImageFormat::Tiff),
        ("pgm", ImageFormat::Pgm),
        ("raw", ImageFormat::Raw),
        ("bmp", ImageFormat::Bmp),
        ("jp2", ImageFormat::Jpeg2000),
        ("wsq", ImageFormat::Wsq),
        ("jpg", ImageFormat::Jpeg),
    ];

    println!("\nComparativo de Formatos (500 DPI, 300x400):");
    for (ext, fmt) in formats {
        let out_file = out_dir.join(format!("sample.{ext}"));
        frame.write_format(&out_file, fmt)?;
        let file_bytes = std::fs::read(&out_file)?;
        let restored = RawFrame::from_file(&out_file)?;
        let q_restored = evaluate_quality(&restored);
        let lossless = restored.sha256() == raw_hash;

        let derived = DerivedManifest {
            schema_version: 1,
            sha256_derived_artifact: sha256_bytes(&file_bytes),
            sha256_raw_frame: raw_hash.clone(),
            format: fmt.to_string(),
            transformations: vec![format!("transcode:png->{fmt}")],
        };
        derived.write_adjacent(&out_file)?;
        let read_back = Manifest::read_adjacent(&out_file)?;

        println!(
            "  {:<5} -> {:6} bytes | lossless={:<5} | qualidade={:3}/100 | custódia_ok={}",
            fmt.to_string(),
            file_bytes.len(),
            lossless,
            *q_restored.score,
            read_back.sha256_raw_frame() == raw_hash
        );
    }

    Ok(())
}
