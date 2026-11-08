use super::*;
use std::{
    io::BufRead,
    net::TcpListener,
    sync::{Arc, Mutex},
};

fn server(responses: Vec<String>) -> (String, std::thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!(
        "http://{}/artifact?access_token=must-not-leak",
        listener.local_addr().unwrap()
    );
    let worker = std::thread::spawn(move || {
        let mut requests = Vec::new();
        for response in responses {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = std::io::BufReader::new(&socket);
            let mut request = String::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" || line.is_empty() {
                    break;
                }
                request.push_str(&line);
            }
            requests.push(request);
            drop(reader);
            socket.write_all(response.as_bytes()).unwrap();
        }
        requests
    });
    (url, worker)
}

fn response(status: u16, headers: &str, body: &str) -> String {
    format!("HTTP/1.1 {status} Test\r\nConnection: close\r\n{headers}\r\n{body}")
}

#[test]
fn transient_statuses_retry_with_retry_after_and_return_the_actual_body() {
    let (url, server) = server(vec![
        response(503, "Content-Length: 0\r\nRetry-After: 2\r\n", ""),
        response(504, "Content-Length: 0\r\n", ""),
        response(200, "Content-Length: 4\r\n", "done"),
    ]);
    let waits = Mutex::new(Vec::new());
    let body = bytes_with(&url, || request(&url, None), &|wait| {
        waits.lock().unwrap().push(wait)
    })
    .unwrap();
    assert_eq!(body, b"done");
    assert_eq!(server.join().unwrap().len(), 3);
    let waits = waits.into_inner().unwrap();
    assert_eq!(waits.len(), 2);
    assert!(waits[0] >= Duration::from_secs(2));
    assert!((Duration::from_millis(500)..=Duration::from_millis(750)).contains(&waits[1]));
}

#[test]
fn persistent_outages_stop_after_four_attempts_and_redact_sensitive_urls() {
    let (url, server) = server(vec![response(503, "Content-Length: 0\r\n", ""); 4]);
    let error = bytes_with(&url, || request(&url, None), &|_| {}).unwrap_err();
    assert!(error.contains("HTTP 503"));
    assert!(!error.contains("access_token") && !error.contains("must-not-leak"));
    assert_eq!(server.join().unwrap().len(), 4);
}

#[test]
fn permanent_errors_and_long_retry_after_do_not_retry_early() {
    for (status, headers) in [
        (404, "Content-Length: 0\r\n"),
        (503, "Content-Length: 0\r\nRetry-After: 120\r\n"),
    ] {
        let (url, server) = server(vec![response(status, headers, "")]);
        assert!(bytes_with(&url, || request(&url, None), &|_| panic!(
            "unexpected retry"
        ))
        .is_err());
        assert_eq!(server.join().unwrap().len(), 1);
    }
}

#[test]
fn interrupted_body_is_restarted_and_never_returned_as_complete() {
    let (url, server) = server(vec![
        response(200, "Content-Length: 8\r\n", "bro"),
        response(200, "Content-Length: 8\r\n", "complete"),
    ]);
    assert_eq!(
        bytes_with(&url, || request(&url, None), &|_| {}).unwrap(),
        b"complete"
    );
    assert_eq!(server.join().unwrap().len(), 2);
}

#[test]
fn read_timeouts_are_bounded_and_reported_without_response_or_url_secrets() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!(
        "http://{}/?code=private-code",
        listener.local_addr().unwrap()
    );
    let sockets = Arc::new(Mutex::new(Vec::new()));
    let held = sockets.clone();
    let worker = std::thread::spawn(move || {
        for _ in 0..4 {
            held.lock().unwrap().push(listener.accept().unwrap().0);
        }
    });
    let call = || {
        classify(
            &url,
            ureq::AgentBuilder::new()
                .timeout_read(Duration::from_millis(25))
                .build()
                .get(&url)
                .call(),
        )
    };
    let error = bytes_with(&url, call, &|_| {}).unwrap_err();
    assert!(error.contains("timed out"), "{error}");
    assert!(!error.contains("private-code"));
    worker.join().unwrap();
    assert_eq!(sockets.lock().unwrap().len(), 4);
}

fn target() -> std::path::PathBuf {
    let mut random = [0; 12];
    getrandom::fill(&mut random).unwrap();
    let name: String = random.iter().map(|n| format!("{n:02x}")).collect();
    let dir = std::env::temp_dir().join(format!("ely-download-{name}"));
    fs::create_dir(&dir).unwrap();
    dir.join("artifact")
}

#[test]
fn an_interrupted_archive_resumes_only_with_the_etag_and_expected_range() {
    let (url, server) = server(vec![
        response(200, "Content-Length: 8\r\nETag: \"v1\"\r\n", "com"),
        response(
            206,
            "Content-Length: 5\r\nContent-Range: bytes 3-7/8\r\nETag: \"v1\"\r\n",
            "plete",
        ),
    ]);
    let to = target();
    // Independent SHA-256 of the eight-byte payload, computed by sha2 rather than the download's signer.
    use sha2::{Digest, Sha256};
    let digest = format!("{:x}", Sha256::digest(b"complete"));
    download_with(&url, 8, &digest, &to, &|_| {}, &|_| {}).unwrap();
    assert_eq!(fs::read(&to).unwrap(), b"complete");
    let requests = server.join().unwrap();
    assert!(requests[1].to_lowercase().contains("range: bytes=3-"));
    assert!(requests[1].to_lowercase().contains("if-range: \"v1\""));
    fs::remove_file(&to).unwrap();
    fs::remove_dir(to.parent().unwrap()).unwrap();
}

#[test]
fn ignored_ranges_restart_and_invalid_hashes_never_replace_existing_files() {
    use sha2::{Digest, Sha256};
    let digest = format!("{:x}", Sha256::digest(b"complete"));
    let (url, server) = server(vec![
        response(200, "Content-Length: 8\r\nETag: \"v1\"\r\n", "com"),
        response(200, "Content-Length: 8\r\n", "complete"),
    ]);
    let to = target();
    download_with(&url, 8, &digest, &to, &|_| {}, &|_| {}).unwrap();
    assert_eq!(fs::read(&to).unwrap(), b"complete");
    server.join().unwrap();
    let (url, worker) = self::server(vec![response(200, "Content-Length: 8\r\n", "corrupt!")]);
    assert!(download_with(&url, 8, &digest, &to, &|_| {}, &|_| {}).is_err());
    assert_eq!(fs::read(&to).unwrap(), b"complete");
    worker.join().unwrap();
    assert_eq!(fs::read_dir(to.parent().unwrap()).unwrap().count(), 1);
    fs::remove_file(&to).unwrap();
    fs::remove_dir(to.parent().unwrap()).unwrap();
}

#[test]
fn wrong_content_ranges_are_rejected_and_partial_archives_are_removed() {
    let (url, worker) = server(vec![
        response(200, "Content-Length: 8\r\nETag: \"v1\"\r\n", "com"),
        response(
            206,
            "Content-Length: 5\r\nContent-Range: bytes 0-4/8\r\n",
            "wrong",
        ),
    ]);
    let to = target();
    assert!(download_with(&url, 8, "unused", &to, &|_| {}, &|_| {})
        .unwrap_err()
        .contains("invalid resume response"));
    assert!(!to.exists());
    assert_eq!(fs::read_dir(to.parent().unwrap()).unwrap().count(), 0);
    worker.join().unwrap();
    fs::remove_dir(to.parent().unwrap()).unwrap();
}

#[test]
fn http_date_retry_after_does_not_get_retried_before_the_server_allows() {
    let date = (chrono::Utc::now() + chrono::Duration::seconds(120)).to_rfc2822();
    let (url, worker) = server(vec![response(
        503,
        &format!("Content-Length: 0\r\nRetry-After: {date}\r\n"),
        "",
    )]);
    assert!(bytes_with(&url, || request(&url, None), &|_| panic!(
        "retried too early"
    ))
    .is_err());
    assert_eq!(worker.join().unwrap().len(), 1);
}
