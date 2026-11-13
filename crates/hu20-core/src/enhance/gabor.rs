//! Block-oriented Gabor enhancement for 500-DPI fingerprint frames.

use crate::{HEIGHT, RawFrame, WIDTH};
#[cfg(feature = "parallel")]
use rayon::prelude::*;

const BLOCK: usize = 16;
const RADIUS: i32 = 4;
const RIDGE_FREQUENCY: f32 = 1.0 / 8.0;

fn orientation(frame: &RawFrame, left: usize, top: usize) -> f32 {
    let mut gxx = 0.0f32;
    let mut gyy = 0.0f32;
    let mut gxy = 0.0f32;
    let pixels = frame.pixels();
    for y in (top.max(1))..(top + BLOCK).min(HEIGHT as usize - 1) {
        for x in (left.max(1))..(left + BLOCK).min(WIDTH as usize - 1) {
            let at = |xx: usize, yy: usize| pixels[yy * WIDTH as usize + xx] as f32;
            let gx = at(x + 1, y - 1) + 2.0 * at(x + 1, y) + at(x + 1, y + 1)
                - at(x - 1, y - 1)
                - 2.0 * at(x - 1, y)
                - at(x - 1, y + 1);
            let gy = at(x - 1, y + 1) + 2.0 * at(x, y + 1) + at(x + 1, y + 1)
                - at(x - 1, y - 1)
                - 2.0 * at(x, y - 1)
                - at(x + 1, y - 1);
            gxx += gx * gx;
            gyy += gy * gy;
            gxy += gx * gy;
        }
    }
    0.5 * (2.0 * gxy).atan2(gxx - gyy) + std::f32::consts::FRAC_PI_2
}

fn kernel(theta: f32) -> Vec<f32> {
    let mut values = Vec::with_capacity(((RADIUS * 2 + 1).pow(2)) as usize);
    let (sin, cos) = theta.sin_cos();
    for y in -RADIUS..=RADIUS {
        for x in -RADIUS..=RADIUS {
            let x_theta = x as f32 * cos + y as f32 * sin;
            let y_theta = -x as f32 * sin + y as f32 * cos;
            let envelope = (-0.5 * (x_theta.powi(2) / 16.0 + y_theta.powi(2) / 9.0)).exp();
            values.push(envelope * (2.0 * std::f32::consts::PI * RIDGE_FREQUENCY * x_theta).cos());
        }
    }
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    for value in &mut values {
        *value -= mean;
    }
    values
}

/// Enhances ridge texture using local Sobel orientation and a 9x9 Gabor kernel.
pub fn enhance_gabor(frame: &RawFrame) -> RawFrame {
    let block_cols = (WIDTH as usize).div_ceil(BLOCK);
    let block_rows = (HEIGHT as usize).div_ceil(BLOCK);
    let orientations: Vec<f32> = (0..(block_cols * block_rows))
        .map(|index| {
            orientation(
                frame,
                (index % block_cols) * BLOCK,
                (index / block_cols) * BLOCK,
            )
        })
        .collect();
    let filters: Vec<Vec<f32>> = orientations.iter().copied().map(kernel).collect();
    let pixels = frame.pixels();
    let w = WIDTH as usize;
    let h = HEIGHT as usize;
    let r = RADIUS as usize;
    let process_row = |y: usize| -> Vec<f32> {
        let mut row = vec![0.0; w];
        if y < r || y + r >= h {
            return row;
        }
        for (x, value) in row.iter_mut().enumerate() {
            if x < r || x + r >= w {
                continue;
            }
            let block_y = (y / BLOCK).min(block_rows - 1);
            let block_x = (x / BLOCK).min(block_cols - 1);
            let block = block_y * block_cols + block_x;
            let filter = &filters[block];
            let mut response = 0.0;
            for ky in -RADIUS..=RADIUS {
                let sy = (y as i32 + ky) as usize;
                for kx in -RADIUS..=RADIUS {
                    let sx = (x as i32 + kx) as usize;
                    let sample = pixels[sy * w + sx] as f32 - 127.5;
                    response += sample
                        * filter[((ky + RADIUS) * (RADIUS * 2 + 1) + (kx + RADIUS)) as usize];
                }
            }
            *value = response;
        }
        row
    };
    #[cfg(feature = "parallel")]
    let response: Vec<f32> = (0..h).into_par_iter().flat_map_iter(process_row).collect();
    #[cfg(not(feature = "parallel"))]
    let response: Vec<f32> = (0..h).flat_map(process_row).collect();
    let (mut min, mut max) = (f32::INFINITY, f32::NEG_INFINITY);
    for y in r..(h - r) {
        let row = &response[y * w..(y + 1) * w];
        for &v in &row[r..(w - r)] {
            min = min.min(v);
            max = max.max(v);
        }
    }
    let scale = if max > min { 255.0 / (max - min) } else { 1.0 };
    let mut out = vec![255u8; w * h];
    for y in r..(h - r) {
        for x in r..(w - r) {
            let idx = y * w + x;
            out[idx] = ((response[idx] - min) * scale).clamp(0.0, 255.0) as u8;
        }
    }
    RawFrame::new(out).expect("Gabor enhancement preserves HU20 frame dimensions")
}
