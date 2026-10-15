use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use std::collections::HashMap;

use crate::Result;
use crate::{Page, card, head, pages, strip_previous, version};

struct Request {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    raw: Vec<u8>,
}

impl Request {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

pub fn run(port: u16, upstream: &str) -> Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", port))?;
    let fonts = card::Fonts::load();
    let pages = pages(true);

    let mut images = HashMap::new();
    for page in &pages {
        let png = card::render(&page.kind, &page.route, &fonts)?;

        images.insert(page.image.clone(), (version(&png), png));
    }

    println!("http://localhost:{port} -> {upstream}, with og tags and images per page");

    thread::scope(|scope| {
        for stream in listener.incoming().flatten() {
            let (upstream, pages, images) = (upstream, &pages, &images);

            scope.spawn(move || {
                if let Err(error) = handle(stream, upstream, pages, images) {
                    eprintln!("prerender: {error}");
                }
            });
        }
    });

    Ok(())
}

fn read_request(stream: &mut TcpStream) -> Result<Request> {
    let mut raw = Vec::new();
    let mut chunk = [0u8; 4096];

    while !raw.windows(4).any(|w| w == b"\r\n\r\n") {
        let read = stream.read(&mut chunk)?;

        if read == 0 || raw.len() > 65536 {
            return Err("incomplete request".into());
        }

        raw.extend_from_slice(&chunk[..read]);
    }

    let head = String::from_utf8_lossy(&raw).to_string();
    let mut lines = head.split("\r\n");
    let mut first = lines.next().unwrap_or("").split(' ');
    let method = first.next().unwrap_or("").to_string();
    let path = first.next().unwrap_or("/").to_string();

    let headers = lines
        .take_while(|line| !line.is_empty())
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .collect();

    Ok(Request {
        method,
        path,
        headers,
        raw,
    })
}

type Images = HashMap<String, (String, Vec<u8>)>;

fn handle(mut client: TcpStream, upstream: &str, pages: &[Page], images: &Images) -> Result<()> {
    let request = read_request(&mut client)?;

    if request
        .header("Upgrade")
        .is_some_and(|v| v.eq_ignore_ascii_case("websocket"))
    {
        return tunnel(client, upstream, &request);
    }

    if request.method != "GET" && request.method != "HEAD" {
        return respond(
            &mut client,
            &request,
            405,
            "text/plain",
            None,
            b"method not allowed",
        );
    }

    let path = request.path.split('?').next().unwrap_or("/");
    let path = if path.len() > 1 {
        path.trim_end_matches('/')
    } else {
        path
    };

    if let Some(name) = path
        .strip_prefix("/og/")
        .and_then(|n| n.strip_suffix(".png"))
    {
        return match images.get(name) {
            Some((_, png)) => respond(&mut client, &request, 200, "image/png", None, png),
            None => respond(&mut client, &request, 404, "text/plain", None, b"not found"),
        };
    }

    if let Some(page) = pages.iter().find(|page| page.route == path) {
        let (_, body, _, _) = fetch(upstream, "/")?;
        let html = strip_previous(&String::from_utf8_lossy(&body));
        let html = html.replace(
            "</head>",
            &format!(
                "  {}\n  </head>",
                head(page, &base(&request), &images[&page.image].0)?
            ),
        );

        return respond(
            &mut client,
            &request,
            200,
            "text/html; charset=utf-8",
            None,
            html.as_bytes(),
        );
    }

    match fetch(upstream, &request.path) {
        Ok((status, body, content_type, location)) => respond(
            &mut client,
            &request,
            status,
            content_type
                .as_deref()
                .unwrap_or("application/octet-stream"),
            location.as_deref(),
            &body,
        ),
        Err(error) => {
            let message = error.to_string();

            respond(
                &mut client,
                &request,
                502,
                "text/plain",
                None,
                message.as_bytes(),
            )
        }
    }
}

/// The public address of the request, so the tags work behind a tunnel.
fn base(request: &Request) -> String {
    let host = request
        .header("X-Forwarded-Host")
        .or_else(|| request.header("Host"))
        .unwrap_or("localhost");

    let scheme = request
        .header("X-Forwarded-Proto")
        .map(str::to_string)
        .or_else(|| {
            let visitor = request.header("Cf-Visitor")?;
            let rest = visitor.split("\"scheme\"").nth(1)?;
            Some(rest.split('"').nth(1)?.to_string())
        })
        .unwrap_or_else(|| {
            let local =
                host.starts_with("localhost") || host.starts_with("127.") || host.starts_with('[');

            if local { "http".into() } else { "https".into() }
        });

    format!("{scheme}://{host}")
}

type Fetched = (u16, Vec<u8>, Option<String>, Option<String>);

fn fetch(upstream: &str, path: &str) -> Result<Fetched> {
    let agent = ureq::AgentBuilder::new().redirects(0).build();

    let response = match agent.get(&format!("{upstream}{path}")).call() {
        Ok(response) => response,
        Err(ureq::Error::Status(_, response)) => response,
        Err(error) => return Err(format!("cannot reach {upstream}: {error}").into()),
    };

    let status = response.status();
    let content_type = response.header("Content-Type").map(str::to_string);
    let location = response.header("Location").map(str::to_string);
    let mut body = Vec::new();

    response.into_reader().read_to_end(&mut body)?;

    Ok((status, body, content_type, location))
}

fn respond(
    client: &mut TcpStream,
    request: &Request,
    status: u16,
    content_type: &str,
    location: Option<&str>,
    body: &[u8],
) -> Result<()> {
    let location = location
        .map(|l| format!("Location: {l}\r\n"))
        .unwrap_or_default();
    let head = format!(
        "HTTP/1.1 {status} OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n{location}Cache-Control: no-cache\r\nConnection: close\r\n\r\n",
        body.len(),
    );

    client.write_all(head.as_bytes())?;

    if request.method != "HEAD" {
        client.write_all(body)?;
    }

    Ok(())
}

/// Hot reload talks over a websocket, so those connections are piped through untouched.
fn tunnel(client: TcpStream, upstream: &str, request: &Request) -> Result<()> {
    let address = upstream.trim_start_matches("http://");
    let mut server =
        TcpStream::connect(address).map_err(|e| format!("cannot reach {upstream}: {e}"))?;

    server.write_all(&request.raw)?;

    let (mut client_read, mut client_write) = (client.try_clone()?, client);
    let mut server_read = server.try_clone()?;

    let up = thread::spawn(move || {
        let _ = std::io::copy(&mut client_read, &mut server);
    });

    let _ = std::io::copy(&mut server_read, &mut client_write);
    let _ = up.join();

    Ok(())
}
