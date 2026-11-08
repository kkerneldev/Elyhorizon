//! Bounded, credential-safe GET retries. OAuth POSTs deliberately do not use these retries.
use crate::i18n::trf;
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    time::Duration,
};

const ATTEMPTS: usize = 4;
const MAX_WAIT: Duration = Duration::from_secs(30);

pub(crate) fn endpoint(url: &str) -> String {
    url::Url::parse(url)
        .map(|mut u| {
            let _ = u.set_username("");
            let _ = u.set_password(None);
            u.set_query(None);
            u.set_fragment(None);
            u.to_string()
        })
        .unwrap_or_else(|_| "invalid URL".into())
}

fn retry_after(value: Option<&str>) -> Option<Duration> {
    let value = value?;
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    let date = chrono::DateTime::parse_from_rfc2822(value).ok()?;
    Some(Duration::from_secs(
        (date.timestamp() - chrono::Utc::now().timestamp()).max(0) as u64,
    ))
}

struct Failure {
    status: Option<u16>,
    message: String,
    retry: bool,
    after: Option<Duration>,
}

fn request(url: &str, range: Option<(u64, &str)>) -> Result<ureq::Response, Failure> {
    let mut request = crate::backend::agent(url)
        .get(url)
        .set("Accept-Encoding", "identity");
    if url.starts_with("https://api.github.com/") {
        if let Ok(token) = std::env::var("HORIZON_GITHUB_TOKEN") {
            if !token.is_empty() {
                request = request.set("Authorization", &format!("Bearer {token}"));
            }
        }
    }
    if let Some((start, validator)) = range {
        request = request
            .set("Range", &format!("bytes={start}-"))
            .set("If-Range", validator);
    }
    classify(url, request.call())
}

fn classify(
    url: &str,
    response: Result<ureq::Response, ureq::Error>,
) -> Result<ureq::Response, Failure> {
    response.map_err(|error| describe(url, &error))
}

fn describe(url: &str, error: &ureq::Error) -> Failure {
    let safe = endpoint(url);
    match error {
        ureq::Error::Status(status, response) => {
            let limited = url.starts_with("https://api.github.com/")
                && crate::github::rate_limited(*status, |h| response.header(h).map(str::to_owned));
            Failure {
                status: Some(*status),
                message: if limited {
                    trf(
                        "GitHub rate limit at {endpoint}. Try again later.",
                        &[("endpoint", &safe)],
                    )
                } else {
                    trf(
                        "Download server {endpoint} returned HTTP {status}.",
                        &[("endpoint", &safe), ("status", &status.to_string())],
                    )
                },
                retry: matches!(*status, 408 | 429 | 500 | 502 | 503 | 504) && !limited,
                after: retry_after(response.header("Retry-After")),
            }
        }
        ureq::Error::Transport(error) => {
            let mut source: Option<&(dyn std::error::Error + 'static)> = Some(error);
            let mut timeout = false;
            while let Some(error) = source {
                timeout |= error.downcast_ref::<std::io::Error>().is_some_and(|e| {
                    matches!(
                        e.kind(),
                        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                    )
                });
                source = error.source();
            }
            Failure {
                status: None,
                message: trf(
                    if timeout {
                        "Download from {endpoint} timed out."
                    } else {
                        "Could not connect to download server {endpoint}. Check your network and proxy settings."
                    },
                    &[("endpoint", &safe)],
                ),
                retry: true,
                after: None,
            }
        }
    }
}

/// Keep the original typed status for GitHub's conditional requests and cache fallback.
pub(crate) fn retry_request(
    url: &str,
    mut call: impl FnMut() -> Result<ureq::Response, Box<ureq::Error>>,
) -> Result<ureq::Response, Box<ureq::Error>> {
    for attempt in 0..ATTEMPTS {
        let result = call();
        if let Err(error) = &result {
            if pause(&describe(url, error.as_ref()), attempt, &std::thread::sleep) {
                continue;
            }
        }
        return result;
    }
    unreachable!()
}

fn pause(error: &Failure, attempt: usize, sleep: &dyn Fn(Duration)) -> bool {
    if !error.retry || attempt + 1 >= ATTEMPTS || error.after.is_some_and(|wait| wait > MAX_WAIT) {
        return false;
    }
    let mut random = [0u8; 2];
    let _ = getrandom::fill(&mut random);
    let jitter = Duration::from_millis(u16::from_ne_bytes(random) as u64 % 251);
    let backoff = Duration::from_millis(250 * (1 << attempt)) + jitter;
    sleep(error.after.unwrap_or_default().max(backoff));
    true
}

pub(crate) fn get(url: &str) -> Result<ureq::Response, String> {
    retry_get(|| request(url, None), &std::thread::sleep)
}

fn retry_get(
    mut call: impl FnMut() -> Result<ureq::Response, Failure>,
    sleep: &dyn Fn(Duration),
) -> Result<ureq::Response, String> {
    for attempt in 0..ATTEMPTS {
        match call() {
            Ok(response) => return Ok(response),
            Err(error) if pause(&error, attempt, sleep) => {}
            Err(error) => return Err(error.message),
        }
    }
    unreachable!()
}

pub(crate) fn small(url: &str) -> Result<Option<Vec<u8>>, String> {
    for attempt in 0..ATTEMPTS {
        let result = request(url, None).and_then(|response| {
            let expected = response
                .header("Content-Length")
                .and_then(|n| n.parse::<usize>().ok());
            let mut body = Vec::new();
            let read = response
                .into_reader()
                .take((1 << 20) + 1)
                .read_to_end(&mut body);
            if body.len() > 1 << 20 {
                return Err(Failure {
                    status: None,
                    message: trf(
                        "Download from {endpoint} exceeded its expected size.",
                        &[("endpoint", &endpoint(url))],
                    ),
                    retry: false,
                    after: None,
                });
            }
            if read.is_ok() && expected.is_none_or(|n| n == body.len()) {
                return Ok(body);
            }
            Err(interrupted(url))
        });
        match result {
            Ok(body) => return Ok(Some(body)),
            Err(error) if error.status == Some(404) => return Ok(None),
            Err(error) if pause(&error, attempt, &std::thread::sleep) => {}
            Err(error) => return Err(error.message),
        }
    }
    unreachable!()
}

pub(crate) fn bytes(url: &str) -> Result<Vec<u8>, String> {
    bytes_with(url, || request(url, None), &std::thread::sleep)
}

fn bytes_with(
    url: &str,
    mut call: impl FnMut() -> Result<ureq::Response, Failure>,
    sleep: &dyn Fn(Duration),
) -> Result<Vec<u8>, String> {
    for attempt in 0..ATTEMPTS {
        let result = call().and_then(|response| {
            let expected = response
                .header("Content-Length")
                .and_then(|s| s.parse::<u64>().ok());
            let mut body = Vec::new();
            if response.into_reader().read_to_end(&mut body).is_ok()
                && expected.is_none_or(|n| n == body.len() as u64)
            {
                return Ok(body);
            }
            Err(interrupted(url))
        });
        match result {
            Ok(body) => return Ok(body),
            Err(error) if pause(&error, attempt, sleep) => {}
            Err(error) => return Err(error.message),
        }
    }
    unreachable!()
}

fn interrupted(url: &str) -> Failure {
    Failure { status: None, message: trf("Download from {endpoint} was interrupted. Retry the installation; verified files are kept.", &[("endpoint", &endpoint(url))]), retry: true, after: None }
}

/// Only a verified complete file is promoted. Range resumes require the same strong ETag
/// and the expected Content-Range; a server ignoring Range restarts from zero.
pub(crate) fn download(
    url: &str,
    size: u64,
    digest: &str,
    to: &Path,
    progress: &impl Fn(f32),
) -> Result<(), String> {
    download_with(url, size, digest, to, progress, &std::thread::sleep)
}

fn download_with(
    url: &str,
    size: u64,
    digest: &str,
    to: &Path,
    progress: &impl Fn(f32),
    sleep: &dyn Fn(Duration),
) -> Result<(), String> {
    let mut random = [0u8; 12];
    getrandom::fill(&mut random).map_err(|e| e.to_string())?;
    let suffix: String = random.iter().map(|n| format!("{n:02x}")).collect();
    let temporary = to.with_extension(format!("part-{suffix}"));
    struct Partial<'a>(&'a Path);
    impl Drop for Partial<'_> {
        fn drop(&mut self) {
            let _ = fs::remove_file(self.0);
        }
    }
    let _cleanup = Partial(&temporary);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    let (mut done, mut total, mut etag) = (0u64, size, None::<String>);
    for attempt in 0..ATTEMPTS {
        let range = etag.as_deref().filter(|_| done > 0).map(|tag| (done, tag));
        let response = match request(url, range) {
            Ok(response) => response,
            Err(error) if pause(&error, attempt, sleep) => continue,
            Err(error) => return Err(error.message),
        };
        let length = response
            .header("Content-Length")
            .and_then(|n| n.parse::<u64>().ok());
        if response.status() == 206 {
            let valid = range.is_some()
                && response
                    .header("Content-Range")
                    .and_then(|s| s.strip_prefix("bytes "))
                    .and_then(|s| s.split_once('/'))
                    .is_some_and(|(span, count)| {
                        let Some((start, end)) = span.split_once('-') else {
                            return false;
                        };
                        let (Ok(start), Ok(end), Ok(count)) = (
                            start.parse::<u64>(),
                            end.parse::<u64>(),
                            count.parse::<u64>(),
                        ) else {
                            return false;
                        };
                        start == done
                            && end >= start
                            && end < count
                            && (total == 0 || count == total)
                            && length.is_none_or(|n| n == end - start + 1)
                    });
            if !valid {
                return Err(trf(
                    "Download server {endpoint} returned an invalid resume response.",
                    &[("endpoint", &endpoint(url))],
                ));
            }
        } else if response.status() == 200 {
            use std::io::Seek;
            file.set_len(0)
                .and_then(|_| file.rewind())
                .map_err(|e| e.to_string())?;
            done = 0;
            etag = response
                .header("ETag")
                .filter(|tag| tag.starts_with('"'))
                .map(str::to_owned);
            if total == 0 {
                total = length.unwrap_or(0);
            }
        } else {
            return Err(trf(
                "Download server {endpoint} returned HTTP {status}.",
                &[
                    ("endpoint", &endpoint(url)),
                    ("status", &response.status().to_string()),
                ],
            ));
        }
        let before = done;
        let mut reader = response.into_reader();
        let mut buf = [0u8; 65536];
        let mut complete = true;
        loop {
            let n = match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => {
                    complete = false;
                    break;
                }
            };
            file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
            done += n as u64;
            if total > 0 && done > total {
                return Err(trf(
                    "Download from {endpoint} exceeded its expected size.",
                    &[("endpoint", &endpoint(url))],
                ));
            }
            progress((done as f32 / total.max(1) as f32).min(1.0) * 0.9);
        }
        complete &= length.is_none_or(|n| done - before == n) && (total == 0 || done == total);
        if !complete {
            let error = interrupted(url);
            if pause(&error, attempt, sleep) {
                continue;
            }
            return Err(error.message);
        }
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        let mut input = fs::File::open(&temporary).map_err(|e| e.to_string())?;
        let mut hash = horizon_release_sign::Sha256::default();
        loop {
            let n = input.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hash.update(&buf[..n]);
        }
        drop(input);
        horizon_release_sign::check(&endpoint(url), digest, &hash.hex())?;
        fs::rename(&temporary, to).map_err(|e| e.to_string())?;
        return Ok(());
    }
    unreachable!()
}

#[cfg(test)]
mod tests;
