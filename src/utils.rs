use std::{
    fs::{self, DirEntry},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn html_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

pub fn extract_first_image_src(html: &str) -> Option<&str> {
    let img_pos = html.find("<img")?;
    let after_img = &html[img_pos..];
    let src_pos = after_img.find("src=\"")?;
    let after_src = &after_img[src_pos + 5..];
    let end = after_src.find('"')?;
    Some(&after_src[..end])
}

pub fn slugify(s: &str) -> String {
    s.trim().to_lowercase().replace(' ', "-")
}

pub fn build_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub fn current_year() -> u32 {
    1970 + (build_timestamp() / 31_557_600) as u32
}

pub fn path_to_url(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn estimate_read_time(body: &str) -> usize {
    let words = body.split_whitespace().count();
    ((words as f64 / 200.0).ceil() as usize).max(1)
}

pub fn format_date(date: &str) -> String {
    const MONTHS: [&str; 12] = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ];
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 { return date.to_string(); }
    let month_idx: usize = parts[1].parse().unwrap_or(0);
    let day: u32 = parts[2].parse().unwrap_or(0);
    if month_idx == 0 || month_idx > 12 || day == 0 { return date.to_string(); }
    format!("{} {} {}", day, MONTHS[month_idx - 1], parts[0])
}

pub fn format_rfc822(date: &str) -> String {
    const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun",
        "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 { return date.to_string(); }
    let Ok(y) = parts[0].parse::<i32>() else { return date.to_string() };
    let Ok(m) = parts[1].parse::<u32>() else { return date.to_string() };
    let Ok(d) = parts[2].parse::<u32>() else { return date.to_string() };
    if m == 0 || m > 12 || d == 0 || d > 31 { return date.to_string(); }
    let t: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let yr = if m < 3 { y - 1 } else { y };
    let dow = ((yr + yr/4 - yr/100 + yr/400 + t[(m-1) as usize] + d as i32).rem_euclid(7)) as usize;
    format!("{}, {:02} {} {} 00:00:00 +0000", DAYS[dow], d, MONTHS[(m-1) as usize], y)
}

pub fn copy_directory(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
    if !src.is_dir() {
        return Err(std::io::Error::other("Source is not a directory"));
    }
    if !dest.is_dir() {
        return Err(std::io::Error::other("Destination is not a directory"));
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name() else { continue };
        let dest_path = dest.join(name);
        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_directory(&path, &dest_path)?;
        } else if path.is_file() {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

pub fn read_all_files_recursive(
    dir: &Path,
    files: &mut Vec<DirEntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            read_all_files_recursive(&path, files)?;
        } else if path.is_file() {
            files.push(entry);
        }
    }
    Ok(())
}
