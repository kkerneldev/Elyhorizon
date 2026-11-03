//! Ely.by authorization-code flow. Browser credentials never enter the launcher.
//! The documented provider requires a registered application and client secret.
use super::*;
use std::io::{BufRead, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Instant;
use url::Url;

const SCOPE: &str = "account_info offline_access minecraft_server_session";
pub const NOT_CONFIGURED: &str = "Ely.by browser sign-in needs a registered Elyhorizon OAuth application. Configure HORIZON_ELY_CLIENT_ID and HORIZON_ELY_CLIENT_SECRET.";
const DEFAULT_REDIRECT: &str = "http://127.0.0.1:38467/oauth/ely";
const APPLICATION_KEY: &str = "ely-oauth-application";

#[derive(serde::Serialize, serde::Deserialize)]
struct Application {
    client_id: String,
    client_secret: String,
    #[serde(default = "default_redirect")]
    redirect_uri: String,
}

fn default_redirect() -> String {
    DEFAULT_REDIRECT.into()
}

pub struct Config {
    client_id: String,
    client_secret: String,
    redirect: Url,
    token_url: String,
    info_url: String,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        Self::load_with(store(), &|key| std::env::var(key).ok())
    }

    fn load_with(
        secrets: &dyn SecretStore,
        environment: &dyn Fn(&str) -> Option<String>,
    ) -> Result<Self, String> {
        let field = |key| environment(key).filter(|s| !s.trim().is_empty());
        let id = field("HORIZON_ELY_CLIENT_ID");
        let secret = field("HORIZON_ELY_CLIENT_SECRET");
        let saved: Option<Application> = if id.is_some() && secret.is_some() {
            None
        } else {
            secrets
                .get(APPLICATION_KEY)?
                .map(|value| {
                    serde_json::from_str(&value)
                        .map_err(|_| crate::i18n::tr(NOT_CONFIGURED).to_owned())
                })
                .transpose()?
        };
        let client_id = id
            .or_else(|| saved.as_ref().map(|a| a.client_id.clone()))
            .ok_or_else(|| crate::i18n::tr(NOT_CONFIGURED).to_owned())?;
        let client_secret = secret
            .or_else(|| saved.as_ref().map(|a| a.client_secret.clone()))
            .ok_or_else(|| crate::i18n::tr(NOT_CONFIGURED).to_owned())?;
        let redirect = field("HORIZON_ELY_REDIRECT_URI")
            .or_else(|| saved.as_ref().map(|a| a.redirect_uri.clone()))
            .unwrap_or_else(default_redirect);
        Ok(Self {
            client_id,
            client_secret,
            redirect: parse_redirect(&redirect)?,
            token_url: "https://account.ely.by/api/oauth2/v1/token".into(),
            info_url: "https://account.ely.by/api/account/v1/info".into(),
        })
    }
}

/// The CLI reads JSON on stdin so the secret never appears in process arguments.
/// Application settings occupy their own keychain item, separate from player sessions.
pub fn configure(input: &str) -> Result<String, String> {
    configure_with(store(), input)
}

fn configure_with(secrets: &dyn SecretStore, input: &str) -> Result<String, String> {
    let invalid = || crate::i18n::tr(NOT_CONFIGURED).to_owned();
    let application: Application = serde_json::from_str(input).map_err(|_| invalid())?;
    if application.client_id.trim().is_empty() || application.client_secret.trim().is_empty() {
        return Err(invalid());
    }
    parse_redirect(&application.redirect_uri)?;
    if !secrets.persistent() {
        return Err(crate::i18n::tr(super::super::NO_KEYCHAIN).to_owned());
    }
    secrets.set(
        APPLICATION_KEY,
        &serde_json::to_string(&application).map_err(|_| invalid())?,
    )?;
    Ok(application.client_id)
}

fn parse_redirect(value: &str) -> Result<Url, String> {
    let invalid = || {
        crate::i18n::tr("Ely.by redirect URI must be an HTTP address on 127.0.0.1 with an explicit port and callback path.").to_owned()
    };
    let url = Url::parse(value).map_err(|_| invalid())?;
    if url.scheme() != "http"
        || url.host_str() != Some("127.0.0.1")
        || url.port().is_none_or(|port| port == 0)
        || url.path() == "/"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid());
    }
    Ok(url)
}

pub struct BrowserFlow {
    config: Config,
    listener: TcpListener,
    state: String,
}

impl BrowserFlow {
    pub fn new(config: Config) -> Result<Self, String> {
        let listener =
            TcpListener::bind(("127.0.0.1", config.redirect.port().expect("validated port")))
                .map_err(|_| {
                    crate::i18n::tr(
                "The Ely.by browser callback port is busy. Close another sign-in and retry.",
            )
                })?;
        listener
            .set_nonblocking(true)
            .map_err(|_| crate::i18n::tr("Could not start Ely.by browser sign-in."))?;
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes)
            .map_err(|_| crate::i18n::tr("Could not create a secure sign-in."))?;
        Ok(Self {
            config,
            listener,
            state: bytes.iter().map(|b| format!("{b:02x}")).collect(),
        })
    }

    pub fn authorization_url(&self, expected: Option<&Account>) -> String {
        let mut url = Url::parse("https://account.ely.by/oauth2/v1").expect("constant URL");
        url.query_pairs_mut()
            .append_pair("client_id", &self.config.client_id)
            .append_pair("redirect_uri", self.config.redirect.as_str())
            .append_pair("response_type", "code")
            .append_pair("scope", SCOPE)
            .append_pair("state", &self.state)
            .append_pair("prompt", "select_account");
        if let Some(account) = expected {
            url.query_pairs_mut()
                .append_pair("login_hint", &account.name);
        }
        url.into()
    }

    pub fn finish(self, cancelled: &dyn Fn() -> bool) -> Result<(Account, Secrets), String> {
        let code = self.wait_code(cancelled, Duration::from_secs(300))?;
        if cancelled() {
            return Err(crate::i18n::tr("Sign-in cancelled.").into());
        }
        let (status, value) = token_request(
            &self.config,
            vec![
                ("grant_type", "authorization_code"),
                ("code", &code),
                ("redirect_uri", self.config.redirect.as_str()),
            ],
        )?;
        let secrets = tokens(status, &value, None)?;
        let account = profile(&self.config, &secrets.minecraft_token)?;
        Ok(with_textures(AUTH_SERVER, (account, secrets)))
    }

    fn wait_code(&self, cancelled: &dyn Fn() -> bool, timeout: Duration) -> Result<String, String> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if cancelled() {
                return Err(crate::i18n::tr("Sign-in cancelled.").into());
            }
            match self.listener.accept() {
                Ok((mut stream, _)) => {
                    let result = callback(&mut stream, &self.config.redirect, &self.state);
                    match result {
                        Callback::Code(code) => {
                            reply(&mut stream, 200);
                            return Ok(code);
                        }
                        Callback::Denied => {
                            reply(&mut stream, 200);
                            return Err(crate::i18n::tr(
                                "Ely.by sign-in was declined in the browser.",
                            )
                            .into());
                        }
                        Callback::Ignore => reply(&mut stream, 400),
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(50))
                }
                Err(_) => {
                    return Err(
                        crate::i18n::tr("Could not receive the Ely.by browser response.").into(),
                    )
                }
            }
        }
        Err(crate::i18n::tr("Ely.by browser sign-in timed out. Try again.").into())
    }
}

enum Callback {
    Code(String),
    Denied,
    Ignore,
}

fn callback(stream: &mut TcpStream, redirect: &Url, state: &str) -> Callback {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let mut reader = std::io::BufReader::new(stream.take(8192));
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return Callback::Ignore;
    }
    let mut parts = line.split_whitespace();
    if parts.next() != Some("GET") {
        return Callback::Ignore;
    }
    let Some(target) = parts
        .next()
        .filter(|s| s.starts_with('/') && !s.starts_with("//"))
    else {
        return Callback::Ignore;
    };
    let Ok(url) = redirect.join(target) else {
        return Callback::Ignore;
    };
    if url.origin() != redirect.origin() || url.path() != redirect.path() {
        return Callback::Ignore;
    }
    let pairs: Vec<_> = url.query_pairs().collect();
    let only = |name: &str| {
        let mut values = pairs
            .iter()
            .filter(|(key, _)| key == name)
            .map(|(_, value)| value.as_ref());
        let first = values.next()?;
        if values.next().is_some() {
            None
        } else {
            Some(first)
        }
    };
    if only("state") != Some(state) {
        return Callback::Ignore;
    }
    if only("error").is_some() {
        return Callback::Denied;
    }
    match only("code").filter(|s| !s.is_empty() && s.len() <= 4096) {
        Some(code) => Callback::Code(code.to_owned()),
        None => Callback::Ignore,
    }
}

fn reply(stream: &mut TcpStream, status: u16) {
    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
    let body =
        crate::i18n::tr("Return to Elyhorizon to finish sign-in. You can close this browser tab.");
    let _ = write!(stream, "HTTP/1.1 {status} Response\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'none'\r\nConnection: close\r\n\r\n{body}", body.len());
}

fn token_request<'a>(
    config: &'a Config,
    mut fields: Vec<(&'a str, &'a str)>,
) -> Result<(u16, Value), String> {
    fields.push(("client_id", &config.client_id));
    fields.push(("client_secret", &config.client_secret));
    send(
        request_to("POST", &config.token_url),
        Some(Body::Form(fields)),
        "Ely.by",
    )
    .map_err(|_| {
        crate::i18n::tr("Could not reach Ely.by. Check your internet connection.").to_owned()
    })
}

fn tokens(status: u16, value: &Value, previous: Option<&Secrets>) -> Result<Secrets, String> {
    if status != 200 {
        return Err(crate::i18n::tr(if matches!(status, 400 | 401 | 403) {
            EXPIRED
        } else {
            "Ely.by is unavailable. Try again later."
        })
        .into());
    }
    let malformed =
        || crate::i18n::tr("Could not read the Ely.by browser sign-in response.").to_owned();
    let access = str_field(value, "access_token")
        .filter(|s| !s.is_empty())
        .ok_or_else(malformed)?;
    if str_field(value, "token_type").is_some_and(|s| !s.eq_ignore_ascii_case("bearer")) {
        return Err(malformed());
    }
    let refresh = str_field(value, "refresh_token")
        .filter(|s| !s.is_empty())
        .or_else(|| previous.map(|s| s.refresh_token.clone()))
        .ok_or_else(malformed)?;
    let seconds = value
        .get("expires_in")
        .and_then(Value::as_u64)
        .filter(|s| *s > 0)
        .ok_or_else(malformed)?;
    Ok(Secrets {
        ely: true,
        ely_oauth: true,
        refresh_token: refresh,
        sign_in_required: false,
        minecraft_token: access,
        minecraft_expires_at: now().saturating_add(seconds),
        xuid: None,
        owns: false,
    })
}

fn profile(config: &Config, token: &str) -> Result<Account, String> {
    let (status, value) = send(
        bearer(request_to("GET", &config.info_url), token),
        None,
        "Ely.by",
    )?;
    if status == 401 || status == 403 {
        return Err(crate::i18n::tr(EXPIRED).into());
    }
    let invalid = || crate::i18n::tr("Could not read the Ely.by profile.").to_owned();
    if status != 200 {
        return Err(invalid());
    }
    let uuid = str_field(&value, "uuid")
        .ok_or_else(invalid)?
        .replace('-', "")
        .to_lowercase();
    let name = str_field(&value, "username").ok_or_else(invalid)?;
    if uuid.len() != 32
        || !uuid.bytes().all(|b| b.is_ascii_hexdigit())
        || !valid_offline_name(&name)
    {
        return Err(invalid());
    }
    Ok(Account {
        uuid: format!("ely:{uuid}"),
        name,
        skin: None,
        cape: None,
    })
}

pub(super) fn refresh(account: &Account, mut secrets: Secrets) -> Check {
    if secrets.sign_in_required {
        return Check::NoAnswer(crate::i18n::tr(EXPIRED).into(), secrets);
    }
    let result = (|| {
        let config = Config::load()?;
        let (status, value) = token_request(
            &config,
            vec![
                ("grant_type", "refresh_token"),
                ("refresh_token", &secrets.refresh_token),
                ("scope", SCOPE),
            ],
        )?;
        let renewed = tokens(status, &value, Some(&secrets))?;
        // Save rotation before a later profile request can fail.
        secrets = renewed;
        let profile = profile(&config, &secrets.minecraft_token)?;
        if profile.uuid != account.uuid {
            return Err(crate::i18n::tr(EXPIRED).to_owned());
        }
        Ok(profile)
    })();
    match result {
        Ok(profile) => {
            let (profile, secrets) = with_textures(AUTH_SERVER, (profile, secrets));
            Check::Owns(profile, secrets)
        }
        Err(e) => {
            secrets.sign_in_required = e == crate::i18n::tr(EXPIRED);
            Check::NoAnswer(e, secrets)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type Requests = Vec<(String, String, String)>;

    #[derive(Default)]
    struct AppStore(MemoryStore);
    impl SecretStore for AppStore {
        fn get(&self, key: &str) -> Result<Option<String>, String> {
            self.0.get(key)
        }
        fn set(&self, key: &str, value: &str) -> Result<(), String> {
            self.0.set(key, value)
        }
        fn delete(&self, key: &str) -> Result<(), String> {
            self.0.delete(key)
        }
        fn persistent(&self) -> bool {
            true
        }
    }

    #[test]
    fn application_configuration_survives_reloads_separately_from_player_accounts() {
        let store = AppStore::default();
        let input = r#"{"client_id":"registered-app","client_secret":"stored-secret"}"#;
        assert_eq!(configure_with(&store, input).unwrap(), "registered-app");
        let loaded = Config::load_with(&store, &|_| None).unwrap();
        assert_eq!(loaded.client_id, "registered-app");
        assert_eq!(loaded.client_secret, "stored-secret");
        assert_eq!(loaded.redirect.as_str(), "http://127.0.0.1:38467/oauth/ely");
        assert!(Accounts::default().accounts.is_empty());
        assert!(!serde_json::to_string(&Accounts::default())
            .unwrap()
            .contains("stored-secret"));
        let overridden = Config::load_with(&store, &|key| match key {
            "HORIZON_ELY_CLIENT_ID" => Some("override-app".into()),
            "HORIZON_ELY_CLIENT_SECRET" => Some("override-secret".into()),
            _ => None,
        })
        .unwrap();
        assert_eq!(overridden.client_id, "override-app");
        assert_eq!(overridden.client_secret, "override-secret");
        assert!(configure_with(&MemoryStore::default(), input).is_err());
        assert!(!configure_with(&store, "invalid JSON with private-data")
            .unwrap_err()
            .contains("private-data"));
    }

    fn api(responses: Vec<(u16, Value)>) -> (Config, std::thread::JoinHandle<Requests>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let mut config = config();
        config.token_url = format!("{base}/token");
        config.info_url = format!("{base}/info");
        let thread = std::thread::spawn(move || {
            let mut requests = Vec::new();
            for (status, value) in responses {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut reader = std::io::BufReader::new(&stream);
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let path = line.split_whitespace().nth(1).unwrap().to_owned();
                let (mut length, mut authorization) = (0, String::new());
                loop {
                    line.clear();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                    if let Some((name, value)) = line.split_once(':') {
                        if name.eq_ignore_ascii_case("content-length") {
                            length = value.trim().parse::<usize>().unwrap();
                        }
                        if name.eq_ignore_ascii_case("authorization") {
                            authorization = value.trim().to_owned();
                        }
                    }
                }
                let mut body = vec![0; length];
                reader.read_exact(&mut body).unwrap();
                requests.push((path, String::from_utf8(body).unwrap(), authorization));
                drop(reader);
                let body = value.to_string();
                write!(stream, "HTTP/1.1 {status} Response\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
            requests
        });
        (config, thread)
    }

    #[test]
    fn authorization_code_is_exchanged_with_the_app_secret_then_loads_the_public_profile() {
        let (config, server) = api(vec![
            (
                200,
                json!({"access_token":"new-access","refresh_token":"refresh-token","expires_in":86400,"token_type":"Bearer"}),
            ),
            (
                200,
                json!({"uuid":"ffc8fdc9-5824-509e-8a57-c99b940fb996","username":"ElyPlayer"}),
            ),
        ]);
        let (status, value) = token_request(
            &config,
            vec![
                ("grant_type", "authorization_code"),
                ("code", "one-use-code"),
                ("redirect_uri", config.redirect.as_str()),
            ],
        )
        .unwrap();
        let secrets = tokens(status, &value, None).unwrap();
        let account = profile(&config, &secrets.minecraft_token).unwrap();
        let requests = server.join().unwrap();
        let form: std::collections::HashMap<_, _> =
            url::form_urlencoded::parse(requests[0].1.as_bytes()).collect();
        assert_eq!(requests[0].0, "/token");
        assert_eq!(form.get("client_id").unwrap(), "test-app");
        assert_eq!(form.get("client_secret").unwrap(), "app-secret");
        assert_eq!(form.get("code").unwrap(), "one-use-code");
        assert_eq!(
            form.get("redirect_uri").unwrap(),
            "http://127.0.0.1:38467/oauth/ely"
        );
        assert_eq!(
            requests[1],
            ("/info".into(), String::new(), "Bearer new-access".into())
        );
        assert_eq!(account.uuid, "ely:ffc8fdc95824509e8a57c99b940fb996");
        let public = serde_json::to_string(&account).unwrap();
        assert!(!public.contains("new-access") && !public.contains("refresh-token"));
    }

    fn config() -> Config {
        Config {
            client_id: "test-app".into(),
            client_secret: "app-secret".into(),
            redirect: parse_redirect("http://127.0.0.1:38467/oauth/ely").unwrap(),
            token_url: String::new(),
            info_url: String::new(),
        }
    }

    fn request(flow: &BrowserFlow, target: &str) -> Callback {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        write!(client, "GET {target} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").unwrap();
        let (mut stream, _) = listener.accept().unwrap();
        callback(&mut stream, &flow.config.redirect, &flow.state)
    }

    #[test]
    fn browser_url_has_session_scope_and_random_state_but_never_the_client_secret() {
        // Bind an ephemeral port for tests, then register that exact loopback URL in the fixture.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut config = config();
        config
            .redirect
            .set_port(Some(listener.local_addr().unwrap().port()))
            .unwrap();
        let flow = BrowserFlow {
            config,
            listener,
            state: "random-state-for-this-test".into(),
        };
        let url = flow.authorization_url(None);
        assert!(!url.contains("app-secret"));
        let url = Url::parse(&url).unwrap();
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().collect();
        assert_eq!(
            pairs.get("scope").unwrap(),
            "account_info offline_access minecraft_server_session"
        );
        assert_eq!(pairs.get("state").unwrap(), "random-state-for-this-test");
        assert_eq!(pairs.get("response_type").unwrap(), "code");
        assert!(
            matches!(request(&flow, "/oauth/ely?state=random-state-for-this-test&code=auth-code"), Callback::Code(c) if c == "auth-code")
        );
        for target in [
            "/oauth/ely?state=wrong&code=stolen",
            "/wrong?state=random-state-for-this-test&code=stolen",
            "/oauth/ely?state=random-state-for-this-test&state=wrong&code=stolen",
            "https://evil.example/oauth/ely?code=stolen",
        ] {
            assert!(matches!(request(&flow, target), Callback::Ignore));
        }
        assert!(matches!(
            request(
                &flow,
                "/oauth/ely?state=random-state-for-this-test&error=access_denied"
            ),
            Callback::Denied
        ));
    }

    #[test]
    fn browser_callback_rejects_nonlocal_redirects_and_cancel_never_waits_for_a_browser() {
        for url in [
            "https://evil.example/oauth/ely",
            "http://localhost:38467/oauth/ely",
            "http://0.0.0.0:38467/oauth/ely",
            "http://127.0.0.1:0/oauth/ely",
            "http://127.0.0.1:38467/",
            "http://127.0.0.1:38467/oauth/ely?state=preset",
        ] {
            assert!(parse_redirect(url).is_err(), "accepted {url}");
        }
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let flow = BrowserFlow {
            config: config(),
            listener,
            state: "unused".into(),
        };
        assert!(flow.wait_code(&|| true, Duration::from_secs(60)).is_err());
        assert!(flow.wait_code(&|| false, Duration::ZERO).is_err());
    }

    #[test]
    fn oauth_refresh_retains_refresh_tokens_when_ely_omits_them_and_never_exposes_errors() {
        let first = tokens(200, &json!({"access_token":"access-1","refresh_token":"refresh-secret","token_type":"Bearer","expires_in":86400}), None).unwrap();
        let renewed = tokens(
            200,
            &json!({"access_token":"access-2","expires_in":86400}),
            Some(&first),
        )
        .unwrap();
        assert_eq!(renewed.refresh_token, "refresh-secret");
        assert_eq!(renewed.minecraft_token, "access-2");
        assert!(renewed.ely_oauth && renewed.ely && !renewed.owns);
        assert!(!format!("{renewed:?}").contains("secret"));
        assert!(
            !tokens(400, &json!({"error_description":"reflected-secret"}), None)
                .unwrap_err()
                .contains("reflected-secret")
        );
        assert!(tokens(
            200,
            &json!({"access_token":"missing-refresh","expires_in":86400}),
            None
        )
        .is_err());
    }
}
