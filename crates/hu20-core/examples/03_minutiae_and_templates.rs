//! Exemplo 03 (Intermediário): Extração de minúcias e serialização de templates ISO/ANSI.
//!
//! Uso:
//!   cargo run -p hu20-core --example 03_minutiae_and_templates

use hu20_core::{
    DPI, FingerPosition, FingerView, HEIGHT, ImpressionType, MinutiaKind, RawFrame, Template,
    TemplateFormat, TemplateMinutia, TemplateMinutiaKind, WIDTH, extract_minutiae,
};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 03. Extração de Minúcias & Templates ISO/ANSI ===");

    let captured_path = PathBuf::from("target/examples/01_basic/capture.png");
    let source_path = if captured_path.exists() {
        captured_path
    } else {
        PathBuf::from("tests/fixtures/probe.png")
    };
    let frame = RawFrame::from_png(&source_path)?;
    let minutiae = extract_minutiae(&frame);
    let endings = minutiae
        .iter()
        .filter(|m| m.kind == MinutiaKind::RidgeEnding)
        .count();
    let bifurcations = minutiae
        .iter()
        .filter(|m| m.kind == MinutiaKind::Bifurcation)
        .count();

    println!(
        "Minúcias extraídas de probe.png: total={} (terminações={}, bifurcações={})",
        minutiae.len(),
        endings,
        bifurcations
    );

    let tpl_minutiae: Vec<TemplateMinutia> = minutiae
        .iter()
        .take(255)
        .map(|m| TemplateMinutia {
            x: m.x.round() as u16,
            y: m.y.round() as u16,
            angle: ((m.angle_deg.rem_euclid(360.0) / 360.0) * 255.0).round() as u8,
            quality: 80,
            kind: match m.kind {
                MinutiaKind::RidgeEnding => TemplateMinutiaKind::RidgeEnding,
                MinutiaKind::Bifurcation => TemplateMinutiaKind::Bifurcation,
            },
        })
        .collect();

    let template = Template {
        product_owner: 0,
        product_type: 0,
        device_id: 0,
        fingers: vec![FingerView {
            position: FingerPosition::RightIndex,
            view: 0,
            impression: ImpressionType::LivePlain,
            quality: 85,
            width: WIDTH as u16,
            height: HEIGHT as u16,
            x_resolution: DPI,
            y_resolution: DPI,
            minutiae: tpl_minutiae,
            singular_points: vec![],
        }],
    };

    let out_dir = PathBuf::from("target/examples/03_templates");
    std::fs::create_dir_all(&out_dir)?;

    let formats = [
        ("iso2005", TemplateFormat::Iso19794_2_2005),
        ("iso2011", TemplateFormat::Iso19794_2_2011),
        ("ansi2004", TemplateFormat::Ansi378_2004),
        ("ansi2009", TemplateFormat::Ansi378_2009),
        ("ansi2009-am1", TemplateFormat::Ansi378_2009Am1),
    ];

    println!("\nComparativo de Perfis de Template:");
    for (label, format) in formats {
        let bytes = template.to_bytes(format)?;
        let decoded = Template::parse(format, &bytes)?;
        let out_file = out_dir.join(format!("probe.{label}.fmr"));
        std::fs::write(&out_file, &bytes)?;
        let view = &decoded.fingers[0];

        println!(
            "  {:<13} -> {:4} bytes | minúcias decodificadas={} | {}x{} @ {} DPI",
            label,
            bytes.len(),
            view.minutiae.len(),
            view.width,
            view.height,
            view.x_resolution
        );
    }

    Ok(())
}
