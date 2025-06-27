use std::fs::File;

use tiny_http::{Method, Request, Response, Server};

fn serve_file(request: Request, path: &str) -> Result<(), ()> {
    let page = File::open(format!("./static/{path}"))
        .map_err(|e| eprintln!("ERROR: couldn't open file `{path}`: {e}"))?;
    let response = Response::from_file(page);

    let _ = request.respond(response);
    Ok(())
}

fn handle_request(request: Request) -> Result<(), ()> {
    let method = request.method();
    let url = request.url().to_string();
    match (method, url.as_str()) {
        // (Method::Get, "/" | "/home" | "/index.html") => serve_file(request, "index.html"),
        // (Method::Get, "/styles.css") => serve_file(request, "styles.css"),
        // (Method::Get, "/rattlesnake") => serve_file(request, "img/rattlesnake.gif"),
        (Method::Get, url) => {
            let path = if url == "/" || url == "/home" || url == "/index.html" {
                "index.html"
            } else {
                url.strip_prefix("/").unwrap_or("")
            };

            if path.contains("..") {
                let response = Response::from_string("Forbidden").with_status_code(403);
                let _ = request.respond(response);
                Err(())
            } else {
                serve_file(request, path)
            }
        }
        _ => Ok(())
    }
}

fn main() -> Result<(), ()> {
    let address = "127.0.0.1:3000";
    let server = Server::http(address)
        .map_err(|e| eprintln!("ERROR: couldn't start server: {e}"))?;

    println!("INFO: started server at {address}");

    for request in server.incoming_requests() {
        let _ = handle_request(request);
    }

    Ok(())
}
