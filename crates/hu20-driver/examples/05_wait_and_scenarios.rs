//! Exemplo 05 (Avançado - Driver): Comparativo de cenários com wait_for_finger (presença + qualidade).
//!
//! Uso:
//!   cargo run -p hu20-driver --example 05_wait_and_scenarios

use hu20_core::evaluate_quality;
use hu20_driver::{Device, MockScenario, MockTransport};
use std::time::Duration;

fn main() {
    println!("=== 05. Comparativo de Cenários com wait_for_finger ===");

    let scenarios = [
        MockScenario::Match,
        MockScenario::Mismatch,
        MockScenario::Degraded,
        MockScenario::Empty,
        MockScenario::Disconnect,
    ];

    for scenario in scenarios {
        let mut dev = Device::mock(MockTransport::new(scenario));
        let mut polls = 0usize;
        let res = dev.wait_for_finger(Duration::from_millis(120), 50, true, |presence, elapsed| {
            polls += 1;
            let _ = (presence, elapsed);
        });

        match res {
            Ok(frame) => {
                let q = evaluate_quality(&frame);
                println!(
                    "  {:<10} -> OK      | polls={polls} | score={:3}/100 | variância={:.1} | sat={:.1}%",
                    format!("{scenario:?}"),
                    *q.score,
                    q.mean_local_variance,
                    q.saturated_ratio * 100.0
                );
            }
            Err(err) => {
                println!(
                    "  {:<10} -> ERRO    | polls={polls} | motivo={err}",
                    format!("{scenario:?}")
                );
            }
        }
    }
}
