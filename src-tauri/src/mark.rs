//! The Limvo mark drawn as pixels, for the notification area icon.
//!
//! The mark is an L whose foot is a meter: it fills from the post outwards as
//! the plan is used, so the icon itself shows how much is left. Drawn in code
//! because it changes with every reading.

/// Edge of the square icon in pixels. Windows scales it for the display.
pub const SIZE: u32 = 32;
/// Samples per pixel along each axis, for smooth edges.
const SUPERSAMPLE: u32 = 4;
const ACCENT: [u8; 3] = [200, 85, 61];
/// The empty part of the meter, as in the app icon.
const TRACK_OPACITY: f64 = 0.28;

/// A rounded rectangle on the 24-unit grid the mark is designed on.
#[derive(Debug, Clone, Copy)]
struct Shape {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
    opacity: f64,
}

impl Shape {
    const fn bar(x: f64, y: f64, width: f64, height: f64, opacity: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            radius: 2.0,
            opacity,
        }
    }

    fn contains(&self, px: f64, py: f64) -> bool {
        let cx = self.x + self.width / 2.0;
        let cy = self.y + self.height / 2.0;
        let dx = ((px - cx).abs() - (self.width / 2.0 - self.radius)).max(0.0);
        let dy = ((py - cy).abs() - (self.height / 2.0 - self.radius)).max(0.0);
        dx * dx + dy * dy <= self.radius * self.radius
    }
}

/// RGBA pixels of the mark with the meter filled to `percent`.
pub fn rgba(percent: Option<f64>) -> Vec<u8> {
    let used = percent.unwrap_or(0.0).clamp(0.0, 100.0) / 100.0;
    let shapes = [
        // The post.
        Shape::bar(4.0, 3.0, 4.0, 18.0, 1.0),
        // The empty meter along the foot.
        Shape::bar(4.0, 17.0, 16.0, 4.0, TRACK_OPACITY),
        // The used part, starting as wide as the post.
        Shape::bar(4.0, 17.0, 4.0 + used * 12.0, 4.0, 1.0),
    ];
    let unit = f64::from(SIZE) / 24.0;
    let step = 1.0 / f64::from(SUPERSAMPLE);
    let samples = f64::from(SUPERSAMPLE * SUPERSAMPLE);

    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let mut coverage = 0.0;
            for sy in 0..SUPERSAMPLE {
                for sx in 0..SUPERSAMPLE {
                    let gx = (f64::from(column) + (f64::from(sx) + 0.5) * step) / unit;
                    let gy = (f64::from(row) + (f64::from(sy) + 0.5) * step) / unit;
                    // Overlapping shapes do not add up: the most opaque one wins.
                    coverage += shapes
                        .iter()
                        .filter(|shape| shape.contains(gx, gy))
                        .map(|shape| shape.opacity)
                        .fold(0.0, f64::max);
                }
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let alpha = (coverage / samples * 255.0).round() as u8;
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

    /// A pixel near the right end of the foot, on the 24-unit grid.
    fn foot_end() -> (u32, u32) {
        let scale = f64::from(SIZE) / 24.0;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let at = |units: f64| (units * scale) as u32;
        (at(18.0), at(19.0))
    }

    #[test]
    fn has_the_expected_size() {
        assert_eq!(rgba(None).len(), (SIZE * SIZE * 4) as usize);
    }

    #[test]
    fn the_meter_fills_with_usage() {
        let (x, y) = foot_end();
        let empty = alpha_at(&rgba(Some(0.0)), x, y);
        let full = alpha_at(&rgba(Some(100.0)), x, y);
        assert_eq!(full, 255);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let track = (TRACK_OPACITY * 255.0).round() as u8;
        assert_eq!(empty, track);
    }

    #[test]
    fn the_post_is_always_drawn() {
        let scale = f64::from(SIZE) / 24.0;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let (x, y) = ((6.0 * scale) as u32, (8.0 * scale) as u32);
        assert_eq!(alpha_at(&rgba(Some(0.0)), x, y), 255);
    }

    #[test]
    fn corners_stay_transparent() {
        let pixels = rgba(Some(50.0));
        assert_eq!(alpha_at(&pixels, 0, 0), 0);
        assert_eq!(alpha_at(&pixels, SIZE - 1, 0), 0);
    }
}
