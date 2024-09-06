use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use infer;
use url_escape::decode;
use crate::utils::file::{serve_file, send_404};

pub fn handle_client(mut stream: TcpStream, root: PathBuf) -> io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    let request = String::from_utf8_lossy(&buffer);
    let request_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    if parts.len() < 2 || parts[0] != "GET" {
        return send_404(&mut stream);
    }

    let decoded_path = decode(parts[1]);
    let requested_path = root.join(decoded_path.trim_start_matches('/'));

    println!("Requested path: {}", requested_path.display());

    if !requested_path.exists() {
        println!("File not found: {}", requested_path.display());
        return send_404(&mut stream);
    }

    let canonical_root = root.canonicalize()?;
    let canonical_requested = requested_path.canonicalize()?;

    if !canonical_requested.starts_with(&canonical_root) {
        return send_404(&mut stream);
    }

    if requested_path.is_dir() {
        return list_directory(&mut stream, &requested_path, parts[1]);
    }

    if requested_path.is_file() {
        return serve_file(&mut stream, &requested_path);
    }

    send_404(&mut stream)
}

fn list_directory(stream: &mut TcpStream, path: &Path, base_url: &str) -> io::Result<()> {
    let current_dir_name = path.file_name().unwrap_or_default().to_string_lossy();
    
    let mut begin_html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8">
        <style>
            .highlight {
                color: red;
            }
        </style>
    </head>
    <body>
    "#.to_string();

    begin_html.push_str(&format!("<h1>Directory listing for {}</h1>", path.display()));

    if base_url.contains("/") {
        let parent_dir;
        if base_url == "/" {
            begin_html.push_str(r#"<a href="/">Parent directory</a><br>"#);
        } else if let Some(pos) = base_url.rfind('/') {
            parent_dir = &base_url[..pos];
            begin_html.push_str(&format!(
                r#"<a href="/{}">Parent directory</a><br>"#,
                parent_dir.trim_start_matches('/')
            ));
        }
    }

    for entry in WalkDir::new(path).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let file_name = entry.file_name().to_string_lossy();
        let file_url = format!("{}/{}", base_url.trim_end_matches('/'), file_name);

        if file_name != current_dir_name {
            begin_html.push_str(&format!(
                r#"<a href="{}">{}</a><br>"#,
                file_url, file_name
            ));
        }
    }

    let end_html = r#"
    </body>
    </html>
    "#;

    let response_body = begin_html + &end_html;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
        response_body.len(),
        response_body
    );

    stream.write(response.as_bytes())?;
    stream.flush()
}
