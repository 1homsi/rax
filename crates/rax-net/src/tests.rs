use rax_async::run_until_stalled;
use rax_reactive::create_root;

use super::{get, post, set_client, MockClient, Response};

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
