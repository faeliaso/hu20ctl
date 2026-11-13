//! Exemplo 08 (Avançado - Sensor Tuning): Comparativo de exposição/ganho no HU20 físico (ou fallback Mock).
//!
//! Uso:
//!   cargo run -p hu20-driver --example 08_sensor_tuning

use hu20_core::{RawFrame, evaluate_quality};
use hu20_driver::{MockScenario, MockTransport, RusbTransport, UsbTransport, list_devices};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 08. Ajuste de Exposição & Ganho do Sensor CMOS HU20 ===");

    let devices = list_devices()?;
    if devices.is_empty() || std::env::var("HU20_MOCK_FIXTURE").is_ok() {
        println!(
            "Nenhum HU20 físico ativo (ou HU20_MOCK_FIXTURE definido): executando simulação Mock."
        );
        let mut mock = MockTransport::new(MockScenario::Match);
        mock.write_bulk(0x01, &[0x04], Duration::from_secs(1))?;
        let mut buf = vec![0u8; 120_000];
        let n = mock.read_bulk(0x82, &mut buf, Duration::from_secs(1))?;
        let frame = RawFrame::new(buf[..n].to_vec())?;
        let q = evaluate_quality(&frame);
        println!(
            "  Mock Match -> score={}/100 | variância={:.1} | sat={:.1}%",
            *q.score,
            q.mean_local_variance,
            q.saturated_ratio * 100.0
        );
        return Ok(());
    }

    let mut transport = RusbTransport::open_first()?;
    println!("Hardware aberto: {:?}", transport.info());

    let presets = [
        ("Exposição baixa (1200, gain 0x06)", 1200_u16, 0x06_u8),
        ("Exposição padrão (2500, gain 0x08)", 2500_u16, 0x08_u8),
        ("Exposição alta (4500, gain 0x10)", 4500_u16, 0x10_u8),
    ];

    for (label, exposure, gain) in presets {
        transport.set_exposure(exposure)?;
        transport.set_gain(gain)?;
        let native_frame = transport.capture_frame()?;
        let core_frame = RawFrame::new(native_frame.into_bytes())?;
        let q = evaluate_quality(&core_frame);
        println!(
            "  {:<36} -> score={:3}/100 | variância={:7.1} | sat={:5.1}% | aceito={}",
            label,
            *q.score,
            q.mean_local_variance,
            q.saturated_ratio * 100.0,
            q.accepted
        );
    }

    let smart_frame = transport.capture_smart_frame()?;
    let smart_core = RawFrame::new(smart_frame.into_bytes())?;
    let q_smart = evaluate_quality(&smart_core);
    println!(
        "  {:<36} -> score={:3}/100 | variância={:7.1} | sat={:5.1}% | aceito={}",
        "Auto-exposição fechada (smart)",
        *q_smart.score,
        q_smart.mean_local_variance,
        q_smart.saturated_ratio * 100.0,
        q_smart.accepted
    );

    Ok(())
}
