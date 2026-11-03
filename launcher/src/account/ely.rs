//! Ely.by's documented Yggdrasil API. Passwords are used once and never persisted.
//! Account IDs are namespaced so an Ely UUID cannot replace a Microsoft identity.
use super::*;
pub mod oauth;

pub const AUTH_SERVER: &str = "https://authserver.ely.by";
pub const TWO_FACTOR: &str = "This Ely.by account requires a two-factor authentication code.";
const EXPIRED: &str = "Your Ely.by sign-in expired. Sign in again.";

/// Pinned official binary, not bundled in the public tree. Verified on every use.
/// https://authlib-injector.yushi.moe/artifact/latest.json (build 56).
const INJECTOR_URL: &str =
    "https://authlib-injector.yushi.moe/artifact/56/authlib-injector-1.2.8.jar";
const INJECTOR_SHA256: &str = "9c7f4343e6c82034958ffb48c14a2cb0c85928be7283103ce17da00c6d5a7b10";

pub fn java_agent(dir: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let path = dir.join("authlib-injector-1.2.8.jar");
    let valid = |bytes: &[u8]| format!("{:x}", Sha256::digest(bytes)) == INJECTOR_SHA256;
    if !std::fs::read(&path).is_ok_and(|bytes| valid(&bytes)) {
        let bytes = crate::backend::get_bytes(INJECTOR_URL)?;
        if !valid(&bytes) {
            return Err(crate::i18n::tr(
                "The Ely.by authentication agent failed its checksum check.",
            )
            .into());
        }
        std::fs::create_dir_all(dir)
            .map_err(|_| crate::i18n::tr("Could not save the Ely.by authentication agent."))?;
        let mut random = [0u8; 12];
        getrandom::fill(&mut random)
            .map_err(|_| crate::i18n::tr("Could not create a secure sign-in."))?;
        let suffix: String = random.iter().map(|b| format!("{b:02x}")).collect();
        let temporary = dir.join(format!("authlib-injector-{suffix}.tmp"));
        std::fs::write(&temporary, bytes)
            .map_err(|_| crate::i18n::tr("Could not save the Ely.by authentication agent."))?;
        if let Err(e) = std::fs::rename(&temporary, &path) {
            let _ = std::fs::remove_file(&temporary);
            if !std::fs::read(&path).is_ok_and(|bytes| valid(&bytes)) {
                return Err(e.to_string());
            }
        }
    }
    Ok(format!("-javaagent:{}=ely.by", path.display()))
}

fn endpoint() -> String {
    AUTH_SERVER.into()
}

fn error(status: u16, value: &Value) -> String {
    use crate::i18n::tr;
    tr(
        if value.get("errorMessage").and_then(Value::as_str)
            == Some("Account protected with two factor auth.")
        {
            TWO_FACTOR
        } else if status == 400 || status == 401 || status == 403 {
            "Ely.by refused the sign-in. Check your login, password and two-factor code."
        } else {
            "Ely.by is unavailable. Try again later."
        },
    )
    .into()
}

fn call(base: &str, action: &str, body: Value) -> Result<(u16, Value), String> {
    send(
        request_to("POST", &format!("{base}/auth/{action}")),
        Some(Body::Json(body)),
        "Ely.by",
    )
    .map_err(|_| {
        crate::i18n::tr("Could not reach Ely.by. Check your internet connection.").to_owned()
    })
}

#[cfg(test)]
fn authenticate(
    base: &str,
    login: &str,
    password: &str,
    totp: &str,
    client: &str,
) -> Result<(Account, Secrets), String> {
    let password = if totp.trim().is_empty() {
        password.to_owned()
    } else {
        format!("{password}:{}", totp.trim())
    };
    let (status, value) = call(
        base,
        "authenticate",
        json!({
            "username": login.trim(), "password": password, "clientToken": client,
            "agent": {"name": "Minecraft", "version": 1},
        }),
    )?;
    if status != 200 {
        return Err(error(status, &value));
    }
    let result = decode(&value, client)?;
    Ok(with_textures(base, result))
}

fn decode(value: &Value, client: &str) -> Result<(Account, Secrets), String> {
    let malformed = || crate::i18n::tr("Could not read the Ely.by profile.").to_owned();
    let token = str_field(value, "accessToken")
        .filter(|s| !s.is_empty())
        .ok_or_else(malformed)?;
    if value.get("clientToken").and_then(Value::as_str) != Some(client) {
        return Err(malformed());
    }
    let profile = value.get("selectedProfile").ok_or_else(malformed)?;
    let uuid = str_field(profile, "id")
        .ok_or_else(malformed)?
        .replace('-', "")
        .to_lowercase();
    let name = str_field(profile, "name").ok_or_else(malformed)?;
    if uuid.len() != 32
        || !uuid.bytes().all(|b| b.is_ascii_hexdigit())
        || !valid_offline_name(&name)
    {
        return Err(malformed());
    }
    Ok((
        Account {
            uuid: format!("ely:{uuid}"),
            name,
            skin: None,
            cape: None,
        },
        Secrets {
            ely_oauth: false,
            refresh_token: client.into(),
            sign_in_required: false,
            minecraft_token: token,
            // Yggdrasil does not promise an expiry field. Refresh on every launch.
            minecraft_expires_at: now() + 7200,
            xuid: None,
            owns: false,
            ely: true,
        },
    ))
}

fn with_textures(base: &str, (mut account, secrets): (Account, Secrets)) -> (Account, Secrets) {
    use base64::Engine;
    let textures = (|| {
        let (status, profile) = send(
            request_to(
                "GET",
                &format!("{base}/session/profile/{}", account.profile_uuid()),
            ),
            None,
            "Ely.by",
        )
        .ok()?;
        if status != 200 {
            return None;
        }
        let property = profile
            .get("properties")?
            .as_array()?
            .iter()
            .find(|p| p.get("name").and_then(Value::as_str) == Some("textures"))?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(property.get("value")?.as_str()?)
            .ok()?;
        serde_json::from_slice::<Value>(&bytes).ok()
    })();
    if let Some(textures) = textures.and_then(|v| v.get("textures").cloned()) {
        let url = |kind: &str| {
            textures
                .get(kind)?
                .get("url")?
                .as_str()
                .filter(|url| texture_url(url))
                .map(str::to_owned)
        };
        account.skin = url("SKIN").map(|url| {
            (
                url,
                if textures
                    .pointer("/SKIN/metadata/model")
                    .and_then(Value::as_str)
                    == Some("slim")
                {
                    "slim"
                } else {
                    "classic"
                }
                .into(),
            )
        });
        account.cape = url("CAPE");
    }
    (account, secrets)
}

/// Only Ely's public texture hosts, never arbitrary URLs from a profile response.
pub fn texture_url(url: &str) -> bool {
    [
        "https://textures.ely.by/",
        "http://textures.ely.by/",
        "https://ely.by/storage/",
        "http://ely.by/storage/",
        "https://skinsystem.ely.by/",
        "http://skinsystem.ely.by/",
        "https://textures.minecraft.net/texture/",
        "http://textures.minecraft.net/texture/",
    ]
    .iter()
    .any(|prefix| url.starts_with(prefix))
}

pub(super) fn refresh(account: &Account, secrets: Secrets) -> Check {
    if secrets.ely_oauth {
        return oauth::refresh(account, secrets);
    }
    refresh_at(&endpoint(), account, secrets)
}

fn refresh_at(base: &str, account: &Account, mut secrets: Secrets) -> Check {
    if secrets.sign_in_required {
        return Check::NoAnswer(crate::i18n::tr(EXPIRED).into(), secrets);
    }
    let result = call(
        base,
        "refresh",
        json!({"accessToken": secrets.minecraft_token, "clientToken": secrets.refresh_token}),
    );
    let (status, value) = match result {
        Ok(result) => result,
        Err(e) => return Check::NoAnswer(e, secrets),
    };
    if status != 200 {
        secrets.sign_in_required = matches!(status, 400 | 401 | 403);
        return Check::NoAnswer(
            if secrets.sign_in_required {
                crate::i18n::tr(EXPIRED).into()
            } else {
                error(status, &value)
            },
            secrets,
        );
    }
    match decode(&value, &secrets.refresh_token) {
        Ok(result) if result.0.uuid == account.uuid => {
            let (profile, renewed) = with_textures(base, result);
            Check::Owns(profile, renewed)
        }
        _ => {
            secrets.sign_in_required = true;
            Check::NoAnswer(crate::i18n::tr(EXPIRED).into(), secrets)
        }
    }
}

/// Revoke this launcher's token only. Other launchers' sessions stay signed in.
pub(super) fn invalidate(secrets: &Secrets) -> Result<(), String> {
    // Ely.by documents no OAuth token-revocation endpoint. OAuth access can be
    // revoked in the account's applications page; local sign-out still forgets it.
    if secrets.ely_oauth {
        return Ok(());
    }
    let (status, value) = call(
        &endpoint(),
        "invalidate",
        json!({"accessToken": secrets.minecraft_token, "clientToken": secrets.refresh_token}),
    )?;
    if status == 200 || status == 204 {
        Ok(())
    } else {
        Err(error(status, &value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, Read, Write};

    const UUID: &str = "ffc8fdc95824509e8a57c99b940fb996";

    fn answer(token: &str) -> Value {
        json!({"accessToken": token, "clientToken": "client", "selectedProfile": {"id": UUID, "name": "ElyPlayer"}})
    }

    /// Real HTTP requests, with all captured bodies returned to the test for protocol assertions.
    fn mock(
        responses: Vec<(u16, Value)>,
    ) -> (String, std::thread::JoinHandle<Vec<(String, Value)>>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let thread = std::thread::spawn(move || {
            let mut requests = Vec::new();
            for (status, response) in responses {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut reader = std::io::BufReader::new(&stream);
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let path = line.split_whitespace().nth(1).unwrap().to_owned();
                let mut length = 0;
                loop {
                    line.clear();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                    if let Some(value) = line.to_lowercase().strip_prefix("content-length:") {
                        length = value.trim().parse::<usize>().unwrap();
                    }
                }
                let mut body = vec![0; length];
                reader.read_exact(&mut body).unwrap();
                requests.push((path, serde_json::from_slice(&body).unwrap_or(Value::Null)));
                drop(reader);
                let body = response.to_string();
                write!(stream, "HTTP/1.1 {status} Response\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
            requests
        });
        (base, thread)
    }

    #[test]
    fn sign_in_sends_totp_loads_slim_skin_and_never_stores_password() {
        use base64::Engine;
        let textures = base64::engine::general_purpose::STANDARD.encode(json!({"textures": {
            "SKIN": {"url": "https://ely.by/storage/skins/abc.png", "metadata": {"model": "slim"}},
            "CAPE": {"url": "https://ely.by/storage/cloaks/abc.png"}
        }}).to_string());
        let (base, thread) = mock(vec![
            (200, answer("access-secret")),
            (
                200,
                json!({"properties": [{"name": "textures", "value": textures}]}),
            ),
        ]);
        let (account, secrets) = authenticate(
            &base,
            " user@example.org ",
            "password-secret",
            "123456",
            "client",
        )
        .unwrap();
        let requests = thread.join().unwrap();
        assert_eq!(requests[0].0, "/auth/authenticate");
        assert_eq!(requests[0].1["username"], "user@example.org");
        assert_eq!(requests[0].1["password"], "password-secret:123456");
        assert_eq!(requests[1].0, format!("/session/profile/{UUID}"));
        assert_eq!(account.skin.as_ref().unwrap().1, "slim");
        assert!(account.cape.is_some());
        let store = MemoryStore::default();
        let mut accounts = Accounts::default();
        accounts.add(&store, account.clone(), &secrets).unwrap();
        assert!(accounts.can_play(&store));
        assert!(!accounts.has_owner(&store));
        let public = serde_json::to_string(&accounts).unwrap();
        assert!(!public.contains("access-secret"));
        let saved = store.get(&account.uuid).unwrap().unwrap();
        assert!(!saved.contains("password-secret") && !saved.contains("123456"));
        assert_eq!(account.profile_uuid(), UUID);
    }

    #[test]
    fn refresh_rotates_access_token_and_refuses_changed_identity() {
        let (account, secrets) = decode(&answer("old"), "client").unwrap();
        let (base, thread) = mock(vec![(200, answer("renewed")), (200, json!({}))]);
        let Check::Owns(_, renewed) = refresh_at(&base, &account, secrets.clone()) else {
            panic!("refresh failed")
        };
        assert_eq!(renewed.minecraft_token, "renewed");
        let requests = thread.join().unwrap();
        assert_eq!(
            requests[0],
            (
                "/auth/refresh".into(),
                json!({"accessToken": "old", "clientToken": "client"})
            )
        );
        let mut changed = answer("wrong-profile");
        changed["selectedProfile"]["id"] = json!("00000000000000000000000000000001");
        let (base, thread) = mock(vec![(200, changed)]);
        assert!(
            matches!(refresh_at(&base, &account, secrets), Check::NoAnswer(_, s) if s.sign_in_required && s.minecraft_token == "old")
        );
        thread.join().unwrap();
    }

    #[test]
    fn two_factor_errors_are_actionable_and_provider_errors_cannot_echo_secrets() {
        let (base, thread) = mock(vec![(
            401,
            json!({"errorMessage": "Account protected with two factor auth."}),
        )]);
        assert_eq!(
            authenticate(&base, "user", "secret", "", "client").unwrap_err(),
            crate::i18n::tr(TWO_FACTOR)
        );
        thread.join().unwrap();
        assert!(!error(401, &json!({"errorMessage": "echoed-secret"})).contains("echoed-secret"));
        let (account, secrets) = decode(&answer("old"), "client").unwrap();
        let (base, thread) = mock(vec![(401, json!({"errorMessage": "expired"}))]);
        assert!(
            matches!(refresh_at(&base, &account, secrets), Check::NoAnswer(_, s) if s.sign_in_required)
        );
        thread.join().unwrap();
    }

    #[test]
    fn providers_with_the_same_uuid_remain_separate_and_ely_never_unlocks_downloads() {
        let (ely, secrets) = decode(&answer("ely-token"), "client").unwrap();
        let microsoft = Account {
            uuid: UUID.into(),
            ..ely.clone()
        };
        let store = MemoryStore::default();
        let mut accounts = Accounts::default();
        accounts
            .add(
                &store,
                microsoft.clone(),
                &Secrets {
                    ely: false,
                    owns: false,
                    ..secrets.clone()
                },
            )
            .unwrap();
        accounts.add(&store, ely.clone(), &secrets).unwrap();
        assert_eq!(accounts.accounts.len(), 2);
        assert_eq!(accounts.selected(), Some(&ely));
        assert_eq!(accounts.find(&format!("ely:{UUID}")), Some(&ely));
        accounts.accounts.swap(0, 1);
        assert_eq!(accounts.find(UUID), Some(&microsoft));
        let endpoints = Endpoints {
            microsoft: String::new(),
            xbox_user: String::new(),
            xsts: String::new(),
            minecraft: "http://127.0.0.1:1".into(),
        };
        assert!(launch(&endpoints, None, &store, &mut accounts).is_err());
        assert!(store.get(UUID).unwrap().is_some());
    }
}
