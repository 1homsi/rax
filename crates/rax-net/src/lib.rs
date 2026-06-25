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

#[cfg(test)]
mod tests;
