use crate::async_rt::run_until_stalled;
use crate::reactive::create_root;

use super::{
    add_interceptor, clear_interceptors, get, get_json, post, set_client, MockClient,
    MultipartForm, Request, Response,
};

fn header_value<'a>(req: &'a Request, name: &str) -> Option<&'a str> {
    req.headers
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
}

#[test]
fn multipart_serializes_fields_and_files() {
    let form = MultipartForm::new()
        .field("title", "Receipt")
        .file("doc", "a.txt", "text/plain", b"hello".to_vec());
    assert_eq!(form.len(), 2);

    let (content_type, body) = form.build();
    assert!(content_type.starts_with("multipart/form-data; boundary="));
    let boundary = content_type.split("boundary=").nth(1).unwrap();

    let text = String::from_utf8(body).unwrap();
    // Each part is introduced by the boundary marker.
    assert_eq!(text.matches(&format!("--{boundary}\r\n")).count(), 2);
    assert!(text.contains("Content-Disposition: form-data; name=\"title\"\r\n\r\nReceipt\r\n"));
    assert!(text.contains(
        "Content-Disposition: form-data; name=\"doc\"; filename=\"a.txt\"\r\nContent-Type: text/plain\r\n\r\nhello\r\n"
    ));
    // Closing boundary.
    assert!(text.ends_with(&format!("--{boundary}--\r\n")));
}

#[test]
fn multipart_boundaries_are_unique() {
    let (ct1, _) = MultipartForm::new().field("a", "1").build();
    let (ct2, _) = MultipartForm::new().field("a", "1").build();
    assert_ne!(ct1, ct2, "each build must mint a fresh boundary");
}

#[test]
fn get_resolves_with_mock_response() {
    set_client(MockClient::new(|req| {
        assert_eq!(req.url, "https://api.test/ping");
        Ok(Response::ok("pong"))
    }));

    let (res, scope) = create_root(|| get("https://api.test/ping"));
    assert!(res.loading());
    run_until_stalled();
    let r = res.data().expect("resolved");
    assert!(r.is_success());
    assert_eq!(r.body, "pong");
    scope.dispose();
}

#[test]
fn post_sends_body_and_errors_propagate() {
    set_client(MockClient::new(|req| {
        if req.body.as_deref() == Some("hi") {
            Ok(Response::ok("got it"))
        } else {
            Err("bad body".to_string())
        }
    }));

    let (ok, scope) = create_root(|| post("https://api.test/echo", "hi"));
    run_until_stalled();
    assert_eq!(ok.data().unwrap().body, "got it");
    scope.dispose();

    let (bad, scope2) = create_root(|| post("https://api.test/echo", "nope"));
    run_until_stalled();
    assert_eq!(bad.error().as_deref(), Some("bad body"));
    scope2.dispose();
}

#[test]
fn interceptor_applies_to_async_get_path() {
    // Regression: a globally-registered interceptor must reach the
    // async/reactive fetch path (send -> dispatch), not just the blocking
    // config helpers.
    clear_interceptors();
    add_interceptor(|_url, headers| {
        headers.push(("Authorization".into(), "Bearer tok".into()));
    });
    set_client(MockClient::new(|req| {
        Ok(Response::ok(
            header_value(req, "Authorization").unwrap_or("MISSING"),
        ))
    }));

    let (res, scope) = create_root(|| get("https://api.test/secure"));
    run_until_stalled();
    assert_eq!(res.data().unwrap().body, "Bearer tok");
    scope.dispose();
    clear_interceptors();
}

#[test]
fn interceptor_applies_to_get_json_path() {
    // Regression: get_json bypassed send(); it must still apply interceptors.
    clear_interceptors();
    add_interceptor(|_url, headers| {
        headers.push(("Authorization".into(), "Bearer tok".into()));
    });
    set_client(MockClient::new(|req| {
        let v = header_value(req, "Authorization").unwrap_or("MISSING");
        Ok(Response::ok(format!("{{\"auth\":{v:?}}}")))
    }));

    let (res, scope) = create_root(|| get_json::<serde_json::Value>("https://api.test/me"));
    run_until_stalled();
    assert_eq!(res.data().unwrap()["auth"].as_str(), Some("Bearer tok"));
    scope.dispose();
    clear_interceptors();
}

#[test]
fn interceptors_apply_in_registration_order() {
    clear_interceptors();
    add_interceptor(|url, _| url.push_str("/a"));
    add_interceptor(|url, _| url.push_str("/b"));
    set_client(MockClient::new(|req| Ok(Response::ok(req.url.clone()))));

    let (res, scope) = create_root(|| get("https://api.test"));
    run_until_stalled();
    assert_eq!(res.data().unwrap().body, "https://api.test/a/b");
    scope.dispose();
    clear_interceptors();
}

#[test]
fn unconfigured_client_reports_error() {
    // A fresh thread: no client set -> the default reports a clear error.
    std::thread::spawn(|| {
        let (res, scope) = create_root(|| get("x"));
        run_until_stalled();
        assert!(res.error().unwrap().contains("no HTTP client"));
        scope.dispose();
    })
    .join()
    .unwrap();
}
