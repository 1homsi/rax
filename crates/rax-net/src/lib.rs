//! HTTP for `rax`, behind a swappable client and returning reactive
//! [`Resource`](rax_async::Resource)s.
//!
//! [`HttpClient`] is the backend trait (a platform implements it over URLSession
//! / a Rust HTTP crate). A thread-local current client is used by [`get`]/[`send`],
//! which kick off the request on the UI executor and hand back a `Resource` that
//! flips from `Loading` to `Ready`/`Failed` when the response arrives.
//!
//! ```
//! use rax_net::{get, set_client, MockClient, Response};
//! use rax_async::run_until_stalled;
//! use rax_reactive::create_root;
//!
//! set_client(MockClient::new(|_req| Ok(Response::ok("pong"))));
//! let (res, scope) = create_root(|| get("https://example.com/ping"));
//! assert!(res.loading());
//! run_until_stalled();
//! assert_eq!(res.data().unwrap().body, "pong");
//! scope.dispose();
//! ```

#![forbid(unsafe_code)]

use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use rax_async::{create_resource, Resource};

/// HTTP method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// GET.
    Get,
    /// POST.
    Post,
    /// PUT.
    Put,
    /// PATCH.
    Patch,
    /// DELETE.
    Delete,
}

/// An HTTP request.
#[derive(Debug, Clone)]
pub struct Request {
    /// Method.
    pub method: Method,
    /// Absolute URL.
    pub url: String,
    /// Header name/value pairs.
    pub headers: Vec<(String, String)>,
    /// Optional request body.
    pub body: Option<String>,
}

impl Request {
    /// A GET request to `url`.
    pub fn get(url: impl Into<String>) -> Request {
        Request {
            method: Method::Get,
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    /// A POST request to `url` with `body`.
    pub fn post(url: impl Into<String>, body: impl Into<String>) -> Request {
        Request {
            method: Method::Post,
            url: url.into(),
            headers: Vec::new(),
            body: Some(body.into()),
        }
    }

    /// Adds a header.
    #[must_use]
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Request {
        self.headers.push((name.into(), value.into()));
        self
    }
}

/// An HTTP response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    /// Status code.
    pub status: u16,
    /// Response body.
    pub body: String,
    /// Response body as raw bytes.
    pub body_bytes: Vec<u8>,
}

impl Response {
    /// A `200 OK` response with `body`.
    pub fn ok(body: impl Into<String>) -> Response {
        let body = body.into();
        Response {
            status: 200,
            body_bytes: body.as_bytes().to_vec(),
            body,
        }
    }

    /// Whether the status is in the 2xx range.
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// A boxed async HTTP result.
pub type ResponseFuture = Pin<Box<dyn Future<Output = Result<Response, String>>>>;

/// The HTTP backend. Implemented by platforms (URLSession, etc.) and by mocks.
pub trait HttpClient {
    /// Sends `request`, resolving to a response or an error message.
    fn send(&self, request: Request) -> ResponseFuture;
}

/// A request handler used by [`MockClient`].
type MockHandler = Rc<dyn Fn(&Request) -> Result<Response, String>>;

/// A synchronous mock client for tests: each request is answered by a closure.
#[derive(Clone)]
pub struct MockClient {
    handler: MockHandler,
}

impl MockClient {
    /// Builds a mock from a response function.
    pub fn new(handler: impl Fn(&Request) -> Result<Response, String> + 'static) -> MockClient {
        MockClient {
            handler: Rc::new(handler),
        }
    }
}

impl HttpClient for MockClient {
    fn send(&self, request: Request) -> ResponseFuture {
        let result = (self.handler)(&request);
        Box::pin(async move { result })
    }
}

struct NotConfigured;
impl HttpClient for NotConfigured {
    fn send(&self, _request: Request) -> ResponseFuture {
        Box::pin(async { Err("no HTTP client configured (call set_client)".to_string()) })
    }
}

thread_local! {
    static CLIENT: std::cell::RefCell<Box<dyn HttpClient>> =
        std::cell::RefCell::new(Box::new(NotConfigured));
}

/// Installs the HTTP client for the current thread.
pub fn set_client(client: impl HttpClient + 'static) {
    CLIENT.with(|c| *c.borrow_mut() = Box::new(client));
}

/// Sends `request` and returns a `Resource` that resolves when it completes.
pub fn send(request: Request) -> Resource<Response> {
    let future = CLIENT.with(|c| c.borrow().send(request));
    create_resource(future)
}

/// Convenience: GET `url` as a `Resource<Response>`.
pub fn get(url: impl Into<String>) -> Resource<Response> {
    send(Request::get(url))
}

/// Convenience: POST `body` to `url` as a `Resource<Response>`.
pub fn post(url: impl Into<String>, body: impl Into<String>) -> Resource<Response> {
    send(Request::post(url, body))
}

/// Execute a GraphQL query or mutation against `endpoint`.
///
/// Builds a JSON body `{"query": "...", "variables": {...}}` and POSTs it with
/// the standard `Content-Type: application/json` header. Returns the full JSON
/// response body as a `Resource<Response>`.
///
/// # Example
/// ```rust,ignore
/// let res = graphql(
///     "https://api.example.com/graphql",
///     r#"query { user(id: "1") { name email } }"#,
///     None,
/// );
/// ```
pub fn graphql(
    endpoint: impl Into<String>,
    query: impl Into<String>,
    variables: Option<String>,
) -> Resource<Response> {
    let endpoint = endpoint.into();
    let query_str = query.into();

    let body = if let Some(vars) = variables {
        format!(r#"{{"query":{:?},"variables":{}}}"#, query_str, vars)
    } else {
        format!(r#"{{"query":{:?}}}"#, query_str)
    };

    let req = Request {
        method: Method::Post,
        url: endpoint,
        headers: vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Accept".to_string(), "application/json".to_string()),
        ],
        body: Some(body),
    };
    send(req)
}

// ---------------------------------------------------------------------------
// WebSocket client
// ---------------------------------------------------------------------------

/// A message received from a WebSocket server.
#[derive(Debug, Clone)]
pub enum WsMessage {
    /// A UTF-8 text frame.
    Text(String),
    /// A binary frame.
    Binary(Vec<u8>),
    /// The connection was closed (no more messages will arrive).
    Close,
}

/// A handle to an active WebSocket connection. Drop to close.
pub struct WsHandle {
    /// Channel to send outgoing messages to the background thread.
    tx: std::sync::mpsc::SyncSender<tungstenite::Message>,
}

impl WsHandle {
    /// Send a text message to the server.
    pub fn send_text(&self, msg: impl Into<String>) {
        let _ = self.tx.send(tungstenite::Message::Text(msg.into().into()));
    }

    /// Send a binary message to the server.
    pub fn send_binary(&self, data: Vec<u8>) {
        let _ = self.tx.send(tungstenite::Message::Binary(data.into()));
    }

    /// Close the connection gracefully.
    pub fn close(self) {
        let _ = self.tx.send(tungstenite::Message::Close(None));
    }
}

/// Connect to a WebSocket server at `url` (must start with `ws://` or `wss://`).
///
/// `on_message` is called from the background thread for each received message.
/// Returns immediately with a [`WsHandle`]. Dropping the handle disconnects.
///
/// ```no_run
/// use rax_net::{connect_ws, WsMessage};
///
/// let handle = connect_ws("ws://echo.websocket.org", |msg| {
///     if let WsMessage::Text(t) = msg {
///         println!("received: {t}");
///     }
/// })
/// .expect("failed to connect");
/// handle.send_text("hello");
/// ```
pub fn connect_ws(
    url: impl Into<String>,
    on_message: impl Fn(WsMessage) + Send + 'static,
) -> Result<WsHandle, String> {
    let url = url.into();
    let (tx, rx) = std::sync::mpsc::sync_channel::<tungstenite::Message>(32);

    std::thread::spawn(move || {
        let (mut socket, _) = match tungstenite::connect(&url) {
            Ok(s) => s,
            Err(e) => {
                on_message(WsMessage::Close);
                let _ = e;
                return;
            }
        };

        loop {
            // Drain any pending outgoing messages first (non-blocking).
            while let Ok(msg) = rx.try_recv() {
                let is_close = matches!(msg, tungstenite::Message::Close(_));
                if socket.send(msg).is_err() || is_close {
                    return;
                }
            }

            // Read the next incoming frame (blocking until one arrives).
            match socket.read() {
                Ok(tungstenite::Message::Text(t)) => on_message(WsMessage::Text(t.to_string())),
                Ok(tungstenite::Message::Binary(b)) => {
                    on_message(WsMessage::Binary(b.to_vec()))
                }
                Ok(tungstenite::Message::Close(_)) | Err(_) => {
                    on_message(WsMessage::Close);
                    return;
                }
                _ => {} // Ping / Pong handled internally by tungstenite
            }
        }
    });

    Ok(WsHandle { tx })
}

// ---------------------------------------------------------------------------
// Query cache — react-query-style deduplication
// ---------------------------------------------------------------------------

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static QUERY_CACHE: RefCell<HashMap<String, Resource<Response>>> =
        RefCell::new(HashMap::new());

    /// Records the wall-clock time when each URL was last fetched and cached.
    static QUERY_TIMESTAMPS: RefCell<HashMap<String, std::time::Instant>> =
        RefCell::new(HashMap::new());
}

/// Returns a cached [`Resource<Response>`] for the given URL.
///
/// The first caller fires an HTTP GET; all subsequent callers with the **same
/// URL** receive the identical `Resource` — the request is never duplicated.
/// The cache is per-thread (all rax work happens on the main thread).
///
/// # Example
/// ```
/// use rax_net::{use_query, set_client, MockClient, Response};
/// use rax_async::run_until_stalled;
/// use rax_reactive::create_root;
///
/// set_client(MockClient::new(|_| Ok(Response::ok("[]"))));
/// let (res, scope) = create_root(|| use_query("https://api.example.com/items"));
/// run_until_stalled();
/// assert!(res.data().is_some());
/// scope.dispose();
/// ```
pub fn use_query(url: impl Into<String>) -> Resource<Response> {
    let url = url.into();
    QUERY_CACHE.with(|cache| {
        if let Some(cached) = cache.borrow().get(&url) {
            return *cached;
        }
        // First caller — fire the request and cache the resource.
        let resource = get(url.clone());
        // Record the timestamp of this fetch.
        QUERY_TIMESTAMPS.with(|t| t.borrow_mut().insert(url.clone(), std::time::Instant::now()));
        cache.borrow_mut().insert(url, resource);
        resource
    })
}

/// Removes the cached entry for `url` so the next [`use_query`] call fires a
/// fresh HTTP GET.
pub fn invalidate_query(url: impl Into<String>) {
    let url = url.into();
    QUERY_CACHE.with(|cache| {
        cache.borrow_mut().remove(&url);
    });
    QUERY_TIMESTAMPS.with(|t| t.borrow_mut().remove(&url));
}

/// Returns a cached [`Resource<Response>`] for the given URL, refetching in
/// the background when the cached entry is older than `stale_after_secs`.
///
/// Pass `0` to never auto-revalidate (always use the cache). Pass
/// `u64::MAX` to always refetch.
///
/// # Example
/// ```
/// use rax_net::{use_query_stale, set_client, MockClient, Response};
/// use rax_async::run_until_stalled;
/// use rax_reactive::create_root;
///
/// set_client(MockClient::new(|_| Ok(Response::ok("[]"))));
/// let (res, scope) = create_root(|| use_query_stale("https://api.example.com/items", 60));
/// run_until_stalled();
/// assert!(res.data().is_some());
/// scope.dispose();
/// ```
pub fn use_query_stale(url: impl Into<String>, stale_after_secs: u64) -> Resource<Response> {
    let url = url.into();

    // A stale_after_secs of 0 means "never revalidate".
    if stale_after_secs != 0 {
        let is_stale = QUERY_TIMESTAMPS.with(|t| {
            t.borrow()
                .get(&url)
                .map(|ts| ts.elapsed().as_secs() > stale_after_secs)
                .unwrap_or(true) // no entry = treat as stale
        });
        if is_stale {
            invalidate_query(url.clone());
        }
    }

    use_query(url)
}

/// Evicts all cache entries that were fetched more than `max_age_secs` ago.
///
/// Call periodically (e.g. on `AppLifecycle::Resumed`) to prevent unbounded
/// memory growth from long-running sessions.
pub fn gc_query_cache(max_age_secs: u64) {
    // Collect URLs that have expired.
    let expired: Vec<String> = QUERY_TIMESTAMPS.with(|t| {
        t.borrow()
            .iter()
            .filter(|(_, ts)| ts.elapsed().as_secs() > max_age_secs)
            .map(|(url, _)| url.clone())
            .collect()
    });
    // Remove both the timestamp and the resource cache entry.
    for url in expired {
        QUERY_CACHE.with(|c| { c.borrow_mut().remove(&url); });
        QUERY_TIMESTAMPS.with(|t| { t.borrow_mut().remove(&url); });
    }
}

// ---------------------------------------------------------------------------
// Server-Sent Events (SSE)
// ---------------------------------------------------------------------------

/// A parsed Server-Sent Event.
#[derive(Debug, Clone)]
pub struct SseEvent {
    /// The event type. Defaults to `"message"` when the stream omits `event:`.
    pub event: String,
    /// The data payload (multi-line `data:` fields are joined with `'\n'`).
    pub data: String,
    /// The optional event id from the `id:` field.
    pub id: Option<String>,
}

/// Connect to a Server-Sent Events endpoint at `url`.
///
/// Spawns a background thread that reads the stream line-by-line and calls
/// `on_event` for every complete event. The thread exits when the server closes
/// the connection or an I/O error occurs. Drop the returned
/// [`std::thread::JoinHandle`] to detach (it will not abort the thread, but the
/// thread will exit on the next failed read once the server closes the stream).
///
/// ```no_run
/// use rax_net::{connect_sse, SseEvent};
///
/// let _handle = connect_sse("https://example.com/events", |ev| {
///     println!("[{}] {}", ev.event, ev.data);
/// });
/// ```
pub fn connect_sse(
    url: impl Into<String>,
    on_event: impl Fn(SseEvent) + Send + 'static,
) -> std::thread::JoinHandle<()> {
    let url = url.into();
    std::thread::spawn(move || {
        let response = match ureq::get(&url)
            .set("Accept", "text/event-stream")
            .set("Cache-Control", "no-cache")
            .call()
        {
            Ok(r) => r,
            Err(_) => return,
        };

        let mut reader = std::io::BufReader::new(response.into_reader());
        let mut event_type = String::from("message");
        let mut data_buf = String::new();
        let mut id_buf: Option<String> = None;

        use std::io::BufRead;
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Err(_) => break,
                _ => {}
            }
            let line = line.trim_end_matches('\n').trim_end_matches('\r');

            if line.is_empty() {
                // Empty line dispatches the buffered event.
                if !data_buf.is_empty() {
                    on_event(SseEvent {
                        event: event_type.clone(),
                        data: data_buf.trim_end_matches('\n').to_string(),
                        id: id_buf.clone(),
                    });
                }
                event_type = "message".to_string();
                data_buf.clear();
                id_buf = None;
            } else if let Some(data) = line.strip_prefix("data:") {
                if !data_buf.is_empty() {
                    data_buf.push('\n');
                }
                data_buf.push_str(data.trim_start());
            } else if let Some(ev) = line.strip_prefix("event:") {
                event_type = ev.trim_start().to_string();
            } else if let Some(id) = line.strip_prefix("id:") {
                id_buf = Some(id.trim_start().to_string());
            }
            // Lines starting with ':' are comments — ignored.
        }
    })
}

#[cfg(test)]
mod tests;
