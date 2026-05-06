use std::fs;
use std::path::Path;
use tiny_http::{Header, Response, Server, StatusCode};

fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("xml") => "application/xml",
        Some("txt") => "text/plain",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

pub fn serve(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("0.0.0.0:{port}");
    let server = Server::http(&addr).map_err(|e| format!("Failed to start server: {e}"))?;
    println!("Serving at http://localhost:{port} — press Ctrl+C to stop");

    let dist_root = Path::new("dist").canonicalize()?;

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let url_path = url.split('?').next().unwrap_or("/");

        let relative = url_path.trim_start_matches('/');
        let candidate = if url_path.ends_with('/') {
            dist_root.join(relative).join("index.html")
        } else {
            dist_root.join(relative)
        };

        let safe_path = candidate.canonicalize().ok().filter(|p| p.starts_with(&dist_root));

        if let Some(path) = safe_path.filter(|p| p.is_file()) {
            let content = fs::read(&path)?;
            let content_type = Header::from_bytes("Content-Type", mime_type(&path)).unwrap();
            request.respond(Response::from_data(content).with_header(content_type))?;
        } else {
            let content = fs::read(dist_root.join("404.html"))
                .unwrap_or_else(|_| b"404 Not Found".to_vec());
            let content_type =
                Header::from_bytes("Content-Type", "text/html; charset=utf-8").unwrap();
            request.respond(
                Response::from_data(content)
                    .with_status_code(StatusCode(404))
                    .with_header(content_type),
            )?;
        }
    }

    Ok(())
}
