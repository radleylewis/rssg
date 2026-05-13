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

pub fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("hello"), "hello");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(html_escape("a & <b> \"c\""), "a &amp; &lt;b&gt; &quot;c&quot;");
    }

    #[test]
    fn test_html_decode() {
        assert_eq!(html_decode("hello"), "hello");
        assert_eq!(html_decode("&amp;"), "&");
        assert_eq!(html_decode("&lt;p&gt;"), "<p>");
        assert_eq!(html_decode("&quot;hi&quot;"), "\"hi\"");
        assert_eq!(html_decode("&#39;"), "'");
        assert_eq!(html_decode("&lt;b&gt;bold&lt;/b&gt;"), "<b>bold</b>");
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("hello"), "hello");
        assert_eq!(xml_escape("a & b"), "a &amp; b");
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
        assert_eq!(xml_escape("a < b & c > d"), "a &lt; b &amp; c &gt; d");
    }

    #[test]
    fn test_json_escape() {
        assert_eq!(json_escape("hello"), "hello");
        assert_eq!(json_escape("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(json_escape("back\\slash"), "back\\\\slash");
        assert_eq!(json_escape("new\nline"), "new\\nline");
        assert_eq!(json_escape("carriage\rreturn"), "carriage\\rreturn");
        assert_eq!(json_escape("tab\there"), "tab\\there");
        assert_eq!(json_escape(""), "");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("  rust  "), "rust");
        assert_eq!(slugify("Already-slug"), "already-slug");
        assert_eq!(slugify("UPPER CASE"), "upper-case");
        assert_eq!(slugify("multiple   spaces"), "multiple---spaces");
    }

    #[test]
    fn test_format_date_valid() {
        assert_eq!(format_date("2024-01-15"), "15 January 2024");
        assert_eq!(format_date("2024-12-01"), "1 December 2024");
        assert_eq!(format_date("1999-06-30"), "30 June 1999");
    }

    #[test]
    fn test_format_date_invalid() {
        assert_eq!(format_date("not-a-date"), "not-a-date");
        assert_eq!(format_date("2024-00-01"), "2024-00-01"); // month 0
        assert_eq!(format_date("2024-13-01"), "2024-13-01"); // month 13
        assert_eq!(format_date("2024-01-00"), "2024-01-00"); // day 0
        assert_eq!(format_date("2024-1"), "2024-1");         // wrong format
    }

    #[test]
    fn test_format_rfc822_known_day() {
        // 2024-01-15 is a Monday
        let result = format_rfc822("2024-01-15");
        assert!(result.starts_with("Mon,"), "expected Mon, got: {result}");
        assert!(result.contains("15 Jan 2024"));
        // 2024-07-04 is a Thursday
        let result = format_rfc822("2024-07-04");
        assert!(result.starts_with("Thu,"), "expected Thu, got: {result}");
    }

    #[test]
    fn test_format_rfc822_invalid() {
        assert_eq!(format_rfc822("not-a-date"), "not-a-date");
        assert_eq!(format_rfc822("2024-00-01"), "2024-00-01");
        assert_eq!(format_rfc822("2024-13-01"), "2024-13-01");
    }

    #[test]
    fn test_estimate_read_time() {
        assert_eq!(estimate_read_time(""), 1); // min 1
        assert_eq!(estimate_read_time(&"word ".repeat(100)), 1);
        assert_eq!(estimate_read_time(&"word ".repeat(200)), 1);
        assert_eq!(estimate_read_time(&"word ".repeat(201)), 2);
        assert_eq!(estimate_read_time(&"word ".repeat(400)), 2);
        assert_eq!(estimate_read_time(&"word ".repeat(401)), 3);
    }

    #[test]
    fn test_extract_first_image_src_found() {
        assert_eq!(
            extract_first_image_src(r#"<img src="/static/photo.jpg" alt="test" />"#),
            Some("/static/photo.jpg")
        );
        assert_eq!(
            extract_first_image_src(r#"<p>text</p><img src="/second.jpg" />"#),
            Some("/second.jpg")
        );
    }

    #[test]
    fn test_extract_first_image_src_multiple_returns_first() {
        assert_eq!(
            extract_first_image_src(r#"<img src="/first.jpg" /><img src="/second.jpg" />"#),
            Some("/first.jpg")
        );
    }

    #[test]
    fn test_extract_first_image_src_not_found() {
        assert_eq!(extract_first_image_src("<p>no images here</p>"), None);
        assert_eq!(extract_first_image_src(""), None);
    }

    #[test]
    fn test_path_to_url() {
        assert_eq!(path_to_url(&PathBuf::from("writing/my-post")), "writing/my-post");
        assert_eq!(path_to_url(&PathBuf::from("a/b/c")), "a/b/c");
        assert_eq!(path_to_url(&PathBuf::from("")), "");
    }
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
