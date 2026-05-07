use crate::build;
use notify::{recommended_watcher, RecursiveMode, Watcher};
use std::fs;
use std::path::Path;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc, Arc,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Response, Server, StatusCode};

const RELOAD_SCRIPT: &str = "<script>\
(function(){var t=null;\
setInterval(function(){\
fetch('/__reload__')\
.then(function(r){return r.text();})\
.then(function(v){if(t===null){t=v;}else if(t!==v){location.reload();}});\
},500);})();\
</script>";

fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("xml") => "application/xml",
        Some("txt") => "text/plain",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn serve(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("Building...");
    build::build_project()?;

    let build_id = Arc::new(AtomicU64::new(now_millis()));
    let build_id_watcher = build_id.clone();

    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();
        let mut watcher = match recommended_watcher(tx) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[WARNING] File watcher failed to start: {e}");
                return;
            }
        };

        for (path, recursive) in &[
            ("pages", RecursiveMode::Recursive),
            ("templates", RecursiveMode::Recursive),
            ("static", RecursiveMode::Recursive),
            ("rssg.toml", RecursiveMode::NonRecursive),
        ] {
            if Path::new(path).exists() {
                let _ = watcher.watch(Path::new(path), *recursive);
            }
        }

        while rx.recv().is_ok() {
            std::thread::sleep(Duration::from_millis(100));
            while rx.try_recv().is_ok() {}

            println!("Change detected, rebuilding...");
            match build::build_project() {
                Ok(()) => {
                    build_id_watcher.store(now_millis(), Ordering::Relaxed);
                    println!("Done.");
                }
                Err(e) => eprintln!("Build error: {e}"),
            }
        }
    });

    let dist_root = Path::new("dist").canonicalize()?;
    let server =
        Server::http(format!("0.0.0.0:{port}")).map_err(|e| format!("Failed to start server: {e}"))?;
    println!("Serving at http://localhost:{port} — press Ctrl+C to stop");

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let url_path = url.split('?').next().unwrap_or("/");

        if url_path == "/__reload__" {
            let id = build_id.load(Ordering::Relaxed).to_string();
            let content_type = Header::from_bytes("Content-Type", "text/plain").unwrap();
            request.respond(Response::from_string(id).with_header(content_type))?;
            continue;
        }

        let relative = url_path.trim_start_matches('/');
        let candidate = {
            let path = dist_root.join(relative);
            if url_path.ends_with('/') || path.is_dir() {
                path.join("index.html")
            } else {
                path
            }
        };

        let safe_path = candidate
            .canonicalize()
            .ok()
            .filter(|p| p.starts_with(&dist_root));

        if let Some(path) = safe_path.filter(|p| p.is_file()) {
            let mime = mime_type(&path);
            let content_type = Header::from_bytes("Content-Type", mime).unwrap();

            if mime.starts_with("text/html") {
                let html = fs::read_to_string(&path)?;
                let html = html.replace("</body>", &format!("{RELOAD_SCRIPT}</body>"));
                request.respond(Response::from_string(html).with_header(content_type))?;
            } else {
                let content = fs::read(&path)?;
                request.respond(Response::from_data(content).with_header(content_type))?;
            }
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
