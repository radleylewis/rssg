use std::{fs, path::Path};

const RASTER_EXTS: &[&str] = &["png", "jpg", "jpeg", "bmp"];

pub fn copy_static_optimized(
    src: &Path,
    dest: &Path,
    max_width: Option<u32>,
    quality: u8,
    converted: &mut Vec<String>,
    prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name() else { continue };
        let name_str = name.to_string_lossy();
        let rel = if prefix.is_empty() { name_str.to_string() } else { format!("{prefix}/{name_str}") };
        let dest_base = dest.join(name);
        if path.is_dir() {
            fs::create_dir_all(&dest_base)?;
            copy_static_optimized(&path, &dest_base, max_width, quality, converted, &rel)?;
        } else {
            let ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()).unwrap_or_default();
            if RASTER_EXTS.contains(&ext.as_str()) {
                let stem = path.file_stem().unwrap_or(name).to_string_lossy();
                let dest_webp = dest.join(format!("{stem}.webp"));
                match encode_webp(&path, &dest_webp, max_width, quality) {
                    Ok(()) => converted.push(rel),
                    Err(e) => {
                        eprintln!("[WARNING] Could not optimize {}: {e} — copying as-is", path.display());
                        fs::copy(&path, &dest_base)?;
                    }
                }
            } else {
                fs::copy(&path, &dest_base)?;
            }
        }
    }
    Ok(())
}

fn encode_webp(src: &Path, dest: &Path, max_width: Option<u32>, quality: u8) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(src)?;
    let img = match max_width {
        Some(max_w) if img.width() > max_w => img.resize(max_w, u32::MAX, image::imageops::FilterType::Lanczos3),
        _ => img,
    };
    let data = if img.color().has_alpha() {
        let rgba = img.into_rgba8();
        webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height()).encode(quality as f32)
    } else {
        let rgb = img.into_rgb8();
        webp::Encoder::from_rgb(rgb.as_raw(), rgb.width(), rgb.height()).encode(quality as f32)
    };
    fs::write(dest, &*data)?;
    Ok(())
}

pub fn rewrite_image_refs(html: &str, converted: &[String]) -> String {
    let mut result = html.to_string();
    for rel in converted {
        if let Some((stem, _)) = rel.rsplit_once('.') {
            result = result.replace(
                &format!("/static/{rel}"),
                &format!("/static/{stem}.webp"),
            );
        }
    }
    result
}
