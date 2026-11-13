//! Exemplo 01 (Básico): Descoberta de scanner HU20, captura (hardware real ou Mock) e exportação.
//!
//! Uso:
//!   cargo run -p hu20-driver --example 01_basic_capture
//!   HU20_MOCK_FIXTURE=match cargo run -p hu20-driver --example 01_basic_capture

use hu20_core::{HEIGHT, WIDTH, evaluate_quality};
use hu20_driver::{Hu20Scanner, MockScenario, MockTransport, list_devices};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let devices = list_devices()?;
    println!("=== 01. Captura Básica (HU20 {WIDTH}x{HEIGHT} @ 500 DPI) ===");
    println!("Dispositivos encontrados: {}", devices.len());
    for dev in &devices {
        println!(
            "  - {:04x}:{:04x} bus {} address {} (model {})",
            dev.vendor_id, dev.product_id, dev.bus_number, dev.address, dev.model
        );
    }

    // Abre hardware real se disponível e não houver mock configurado; caso contrário usa MockScenario::Match.
    let use_mock = std::env::var("HU20_MOCK_FIXTURE").is_ok() || devices.is_empty();
    let mut scanner = if use_mock {
        let scenario = std::env::var("HU20_MOCK_FIXTURE")
            .ok()
            .and_then(|s| s.parse::<MockScenario>().ok())
            .unwrap_or(MockScenario::Match);
        println!("Modo: MockTransport ({scenario:?})");
        Hu20Scanner::new_mock(MockTransport::new(scenario))
    } else {
        println!("Modo: Hardware físico HU20");
        Hu20Scanner::open_first()?
    };

    let frame = scanner.capture_smart()?;
    let quality = evaluate_quality(&frame);

    let out_dir = PathBuf::from("target/examples/01_basic");
    std::fs::create_dir_all(&out_dir)?;
    let png_path = out_dir.join("capture.png");
    let pgm_path = out_dir.join("capture.pgm");
    frame.write_png(&png_path)?;
    frame.write_pgm(&pgm_path)?;

    println!("Frame capturado: {} bytes", frame.pixels().len());
    println!("SHA-256: {}", frame.sha256());
    println!(
        "Qualidade: {}/100 | aceito={} | variância={:.1} | saturação={:.1}%",
        *quality.score,
        quality.accepted,
        quality.mean_local_variance,
        quality.saturated_ratio * 100.0
    );
    println!("Arquivos salvos em: {}", out_dir.display());
    Ok(())
}
