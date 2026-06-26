//! URLSession-backed HTTP client for iOS: `ureq` on a background thread with
//! a `futures::channel::oneshot` bridge back to the UI executor.

use futures::channel::oneshot;
use crate::net::{HttpClient, Method, Request, Response, ResponseFuture};

/// An HTTP client that runs requests on a background thread via `ureq` and
/// delivers results to the `rax` async executor via a oneshot channel.
pub(crate) struct UreqClient;

impl HttpClient for UreqClient {
    fn send(&self, request: Request) -> ResponseFuture {
        let (tx, rx) = oneshot::channel::<Result<Response, String>>();
        std::thread::spawn(move || {
            let result = execute(request);
            // Receiver may have been dropped if the resource was disposed.
            let _ = tx.send(result);
        });
        Box::pin(async move {
            rx.await.unwrap_or(Err("request cancelled".to_string()))
        })
    }
}

fn execute(req: Request) -> Result<Response, String> {
    let agent = ureq::AgentBuilder::new().build();
    let mut builder = match req.method {
        Method::Get => agent.get(&req.url),
        Method::Post => agent.post(&req.url),
        Method::Put => agent.put(&req.url),
        Method::Patch => agent.patch(&req.url),
        Method::Delete => agent.delete(&req.url),
    };
    for (k, v) in &req.headers {
        builder = builder.set(k, v);
    }
    let resp = if let Some(body) = req.body {
        builder
            .set("Content-Type", "application/json")
            .send_string(&body)
            .map_err(|e| e.to_string())?
    } else {
        builder.call().map_err(|e| e.to_string())?
    };
    let status = resp.status();
    let mut body_bytes = Vec::new();
    use std::io::Read;
    resp.into_reader()
        .read_to_end(&mut body_bytes)
        .map_err(|e| e.to_string())?;
    let body = String::from_utf8_lossy(&body_bytes).to_string();
    Ok(Response {
        status,
        body,
        body_bytes,
    })
}
