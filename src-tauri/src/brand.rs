//! The provider's own mark, when the user supplied one.
//!
//! Brand files are not part of the project. A user who wants a provider's
//! official logo puts it at `public/brand/<provider>.svg` before building, and
//! it ends up in the bundled interface. The widget windows load it directly;
//! for the notification area it is rasterised here.

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use tauri::AppHandle;
use tauri::image::Image;

/// The bundled SVG for `provider`, if there is one.
pub fn svg(app: &AppHandle, provider: &str) -> Option<Vec<u8>> {
    let path = format!("brand/{provider}.svg");
    if let Some(asset) = app.asset_resolver().get(path.clone()) {
        return Some(asset.bytes);
    }
    // In development the interface is served by Vite, so read the source folder.
    #[cfg(debug_assertions)]
    {
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("public")
            .join(&path);
        if let Ok(bytes) = std::fs::read(source) {
            return Some(bytes);
        }
    }
    None
}

/// Draws an SVG into a square icon of `size` pixels, centred and scaled to fit.
pub fn icon(svg: &[u8], size: u32) -> Option<Image<'static>> {
    let tree = Tree::from_data(svg, &Options::default()).ok()?;
    let mut pixmap = Pixmap::new(size, size)?;
    let source = tree.size();
    #[allow(clippy::cast_precision_loss)]
    let edge = size as f32;
    let scale = (edge / source.width()).min(edge / source.height());
    let offset_x = (edge - source.width() * scale) / 2.0;
    let offset_y = (edge - source.height() * scale) / 2.0;
    let transform = Transform::from_scale(scale, scale).post_translate(offset_x, offset_y);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // The icon wants straight alpha; the pixmap stores premultiplied colour.
    let rgba = pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let colour = pixel.demultiply();
            [colour.red(), colour.green(), colour.blue(), colour.alpha()]
        })
        .collect();
    Some(Image::new_owned(rgba, size, size))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterises_a_simple_mark() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10" fill="#c8553d"/></svg>"##;
        let image = icon(svg, 32).expect("renders");
        assert_eq!((image.width(), image.height()), (32, 32));
        let centre = ((16 * 32 + 16) * 4) as usize;
        assert_eq!(&image.rgba()[centre..centre + 4], &[200, 85, 61, 255]);
    }

    #[test]
    fn rejects_something_that_is_not_svg() {
        assert!(icon(b"not an image", 32).is_none());
    }
}
