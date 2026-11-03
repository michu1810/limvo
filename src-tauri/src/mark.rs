//! The Headroom mark drawn as pixels, for the notification area icon.
//!
//! The crossbar of the H is a gauge: it sits low on a fresh week and rises as
//! the plan fills up, so the icon itself shows how much headroom is left.
//! Drawn in code because it changes with every reading.

/// Edge of the square icon in pixels. Windows scales it for the display.
pub const SIZE: u32 = 32;
/// Samples per pixel along each axis, for smooth edges.
const SUPERSAMPLE: u32 = 4;
const ACCENT: [u8; 3] = [200, 85, 61];

/// A rounded rectangle on the 24-unit grid the mark is designed on.
#[derive(Debug, Clone, Copy)]
struct Shape {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
}

impl Shape {
    fn contains(&self, px: f64, py: f64) -> bool {
        let cx = self.x + self.width / 2.0;
        let cy = self.y + self.height / 2.0;
        let dx = ((px - cx).abs() - (self.width / 2.0 - self.radius)).max(0.0);
        let dy = ((py - cy).abs() - (self.height / 2.0 - self.radius)).max(0.0);
        dx * dx + dy * dy <= self.radius * self.radius
    }
}

/// RGBA pixels of the mark with the crossbar at `percent` (0 low, 100 high).
pub fn rgba(percent: Option<f64>) -> Vec<u8> {
    let used = percent.unwrap_or(0.0).clamp(0.0, 100.0) / 100.0;
    let shapes = [
        Shape {
            x: 4.0,
            y: 3.0,
            width: 4.0,
            height: 18.0,
            radius: 2.0,
        },
        Shape {
            x: 16.0,
            y: 3.0,
            width: 4.0,
            height: 18.0,
            radius: 2.0,
        },
        Shape {
            x: 6.0,
            y: 16.0 - used * 12.0,
            width: 12.0,
            height: 4.0,
            radius: 2.0,
        },
    ];
    let unit = f64::from(SIZE) / 24.0;
    let step = 1.0 / f64::from(SUPERSAMPLE);
    let samples = f64::from(SUPERSAMPLE * SUPERSAMPLE);

    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let mut covered = 0u32;
            for sy in 0..SUPERSAMPLE {
                for sx in 0..SUPERSAMPLE {
                    let gx = (f64::from(column) + (f64::from(sx) + 0.5) * step) / unit;
                    let gy = (f64::from(row) + (f64::from(sy) + 0.5) * step) / unit;
                    if shapes.iter().any(|shape| shape.contains(gx, gy)) {
                        covered += 1;
                    }
                }
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let alpha = (f64::from(covered) / samples * 255.0).round() as u8;
            pixels.extend_from_slice(&[ACCENT[0], ACCENT[1], ACCENT[2], alpha]);
        }
    }
    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha_at(pixels: &[u8], x: u32, y: u32) -> u8 {
        pixels[((y * SIZE + x) * 4 + 3) as usize]
    }

    #[test]
    fn has_the_expected_size() {
        assert_eq!(rgba(None).len(), (SIZE * SIZE * 4) as usize);
    }

    #[test]
    fn the_crossbar_rises_with_usage() {
        // Centre column, between the posts.
        let x = SIZE / 2;
        let low = rgba(Some(0.0));
        let high = rgba(Some(100.0));
        let row = |units: f64| {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let row = (units * f64::from(SIZE) / 24.0) as u32;
            row
        };
        assert_eq!(alpha_at(&low, x, row(18.0)), 255);
        assert_eq!(alpha_at(&low, x, row(6.0)), 0);
        assert_eq!(alpha_at(&high, x, row(6.0)), 255);
        assert_eq!(alpha_at(&high, x, row(18.0)), 0);
    }

    #[test]
    fn corners_stay_transparent() {
        let pixels = rgba(Some(50.0));
        assert_eq!(alpha_at(&pixels, 0, 0), 0);
        assert_eq!(alpha_at(&pixels, SIZE - 1, SIZE - 1), 0);
    }
}
