use std::fs;
use std::io::{self, Write};
use std::net::TcpStream;
use std::path::Path;
use infer;

pub fn serve_file(stream: &mut TcpStream, path: &Path) -> io::Result<()> {
    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        return send_404(stream);
    }

    let file_content = fs::read(path)?;

    let mime_type = infer::get_from_path(path)
        .ok()
        .flatten()
        .map(|kind| kind.mime_type())
        .unwrap_or_else(|| match path.extension().and_then(|ext| ext.to_str()) {
            Some("html") => "text/html",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("mp4") => "video/mp4",
            Some("gif") => "image/gif",
            Some("pdf") => "application/pdf",
            Some("txt") | Some("gitignore") => "text/plain",
            Some("rs") => "text/rs",
            _ => "text/plain",
        });

    println!("Serving file: {} with MIME type: {}", path.display(), mime_type);

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {}\r\nContent-Disposition: inline\r\n\r\n",
        file_content.len(),
        mime_type
    );

    stream.write(response.as_bytes())?;
    stream.write(&file_content)?;
    stream.flush()
}

pub fn send_404(stream: &mut TcpStream) -> io::Result<()> {
    let body = "<html><body><h1>404 - Not Found</h1></body></html>";
    let response = format!(
        "HTTP/1.1 404 NOT FOUND\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write(response.as_bytes())?;
    stream.flush()
}
