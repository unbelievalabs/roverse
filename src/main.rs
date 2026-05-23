use std::{env, net::SocketAddr};

use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::{
        HeaderMap, Request, StatusCode, Uri,
        header::{HOST, HeaderName},
    },
    response::{IntoResponse, Redirect, Response},
    routing::any,
};
use reqwest::Client;

const PROXY_SECRET_HEADER: &str = "x-proxy-secret";

#[derive(Clone)]
struct AppState {
    client: Client,
    secret: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = env::var("ROVERSE_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_owned())
        .parse::<SocketAddr>()?;

    let client = Client::builder().build()?;
    let secret = env::var("ROVERSE_SECRET")
        .ok()
        .filter(|secret| !secret.is_empty());
    let app = Router::new()
        .route("/", any(root))
        .route("/{subdomain}", any(proxy_root))
        .route("/{subdomain}/", any(proxy_root))
        .route("/{subdomain}/{*path}", any(proxy_path))
        .with_state(AppState { client, secret });

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn root() -> Redirect {
    Redirect::temporary("https://www.roblox.com")
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn proxy_root(
    State(state): State<AppState>,
    Path(subdomain): Path<String>,
    request: Request<Body>,
) -> Response {
    proxy(state, subdomain, request).await
}

async fn proxy_path(
    State(state): State<AppState>,
    Path((subdomain, _path)): Path<(String, String)>,
    request: Request<Body>,
) -> Response {
    proxy(state, subdomain, request).await
}

async fn proxy(state: AppState, subdomain: String, request: Request<Body>) -> Response {
    if !is_valid_subdomain(&subdomain) {
        return (StatusCode::BAD_REQUEST, "invalid Roblox subdomain").into_response();
    }

    if !is_authorized(&state, request.headers()) {
        return (StatusCode::UNAUTHORIZED, "missing or invalid proxy secret").into_response();
    }

    let url = upstream_url(&subdomain, request.uri());
    let (parts, body) = request.into_parts();
    let mut outbound = state
        .client
        .request(parts.method, url)
        .body(reqwest::Body::wrap_stream(body.into_data_stream()));

    for (name, value) in parts.headers.iter() {
        if should_forward_request_header(name) {
            outbound = outbound.header(name, value);
        }
    }

    match outbound.send().await {
        Ok(upstream) => response_from_upstream(upstream),
        Err(error) => (
            StatusCode::BAD_GATEWAY,
            format!("upstream request failed: {error}"),
        )
            .into_response(),
    }
}

fn is_authorized(state: &AppState, headers: &HeaderMap) -> bool {
    let Some(secret) = &state.secret else {
        return true;
    };

    headers
        .get(PROXY_SECRET_HEADER)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == secret)
}

fn upstream_url(subdomain: &str, uri: &Uri) -> String {
    let mut url = format!("https://{subdomain}.roblox.com");
    let path = upstream_path(subdomain, uri);

    if !path.is_empty() {
        url.push('/');
        url.push_str(&path);
    }

    if let Some(query) = uri.query() {
        url.push('?');
        url.push_str(query);
    }

    url
}

fn upstream_path(subdomain: &str, uri: &Uri) -> String {
    let Some(mut path) = uri.path().strip_prefix('/') else {
        return String::new();
    };

    if path == subdomain {
        return String::new();
    }

    let Some(rest) = path.strip_prefix(subdomain) else {
        return String::new();
    };

    path = rest.strip_prefix('/').unwrap_or(rest);
    path.to_owned()
}

fn response_from_upstream(upstream: reqwest::Response) -> Response {
    let status = upstream.status();
    let headers = upstream.headers().clone();
    let mut response = Response::new(Body::from_stream(upstream.bytes_stream()));
    *response.status_mut() = status;
    copy_response_headers(&headers, response.headers_mut());
    response
}

fn copy_response_headers(source: &HeaderMap, target: &mut HeaderMap) {
    for (name, value) in source.iter() {
        if should_forward_response_header(name) {
            target.append(name, value.clone());
        }
    }
}

fn should_forward_request_header(name: &HeaderName) -> bool {
    name != HOST && !is_hop_by_hop_header(name)
}

fn should_forward_response_header(name: &HeaderName) -> bool {
    !is_hop_by_hop_header(name)
}

fn is_hop_by_hop_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "proxy-connection"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}

fn is_valid_subdomain(subdomain: &str) -> bool {
    !subdomain.is_empty()
        && subdomain.len() <= 63
        && !subdomain.starts_with('-')
        && !subdomain.ends_with('-')
        && subdomain
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}
