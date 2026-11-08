# Elyhorizon 0.2.2 local stabilization and verification

All changes are on local `main`, starting at `a7d5f71bab`. At the initial check, both
`origin/main` and the live remote `refs/heads/main` had that same commit. The previous
`sync/upstream-2026-11-08` branch remains separate; it was not merged into `main`.
The agent issued no commits, pushes, tags, PRs, workflow dispatches or release operations.
During verification, externally created commits `598022bdde` and `e7d1d7fd00` appeared on
`main`/`origin/main`; they were preserved. The latter changes the validation redirect port
to 38467. The final follow-up fixes remain in the local working tree on that same branch.

## Authentication findings

The official [discovery document](https://account.ely.by/.well-known/openid-configuration)
advertises Authorization Code, S256, token endpoint authentication `none`, and authorization
response issuer identification. The older [OAuth guide](https://docs.ely.by/en/oauth.html)
still documents the flat `uuid` / `username` account-info response, but is incomplete about
public Desktop registrations.

Live checks on 2026-11-08 found:

| Check | Observed result |
| --- | --- |
| Public-client PKCE support at the provider | Advertised; the existing control registration passed secret-free invalid-code exchange checks |
| Exact requested ID `elyhorizondesktop` | `/api/oauth2/v1/validate`: HTTP 401 `invalid_client` |
| Historical ID `elyherizondesktop` | S256 + loopback validation succeeded |
| Published Windows 0.2.2 ZIP | SHA-256 matched its release checksum file |
| Published 0.2.2 EXE with fresh homes and no runtime Ely variables | Embedded `elyherizondesktop`, S256, `client_secret_required=false` |

The published artifact was not missing its public configuration. The spelling discrepancy
must be resolved against the actual **Client ID**, rather than an application's display name.
The existing repository variable is reused; no new variable or secret is required. New builds
use its exact value and the preflight refuses a registration the provider rejects. The code
does not substitute the historical ID automatically.

The PKCE verifier and state were already generated independently from cryptographic randomness.
The listener already bound 127.0.0.1 before the browser was opened, using an ephemeral port to
avoid conflicts. Those mechanisms are retained. Changes harden complete-header time limits,
Host validation, duplicate parameters, issuer/state matching, ambiguous code/error responses,
and cancellation/denial handling. Browser success is now sent after profile validation and
account persistence, rather than immediately after receiving a code.

A confirmed validation defect rejected short player names because the parser reused the
offline-name minimum of three characters. The provider profile parser now accepts a valid
nonempty ASCII Minecraft player name of up to 16 characters. This is a code defect; without
a failing user's actual response, it cannot be claimed as the cause of every reported
"Could not read the Ely.by profile" error. HTTP statuses, malformed responses, connection
errors and timeouts now identify the operation without echoing provider bodies or credentials.

Successful saved Ely sign-in restores the existing window once. Windows uses native restoration
and foreground APIs with taskbar attention if foreground activation is denied, without GPUI's
simulated keyboard input. macOS explicitly deminiaturizes its existing NSWindow and requests supported application/window
activation. Linux requests activation and attention through its window backend. Wayland/compositor rules can refuse focus; forced focus is not promised. Native GUI
behavior remains a manual platform check.

## Account storage and launching

Ely launch authorization remains separate from Microsoft ownership. A valid Ely session may
start Horizon/Java without Microsoft credentials; Microsoft accounts still pass their existing
ownership checks. Ely provider errors now survive the launch gate instead of being converted
into a Microsoft-only ownership message. Account selection and the namespaced identity remain
unchanged after a failed renewal.

Windows/macOS retain their OS credential stores. Linux/FreeBSD fallback storage now writes
`HZSECv2`, using a random owner-only `account-secrets.key` rather than a predictable machine-ID
key. `HZSECv1` remains readable and is migrated on the next write. Keep the key and encrypted
file private; backing up both carries access to the accounts. No credentials belong in this
repository or in an installer.

## Installation and network reliability

The installation path reads:

1. GitHub release metadata, with the existing cached/mirror fallback.
2. Signed release checksums and signatures, then the platform game archive/CDN redirect.
3. Pinned Mojang version manifests, client/server JARs, library artifacts and asset indexes.
4. Asset objects from `resources.download.minecraft.net`, and official Java runtime manifests/files.

The live release-list API, GitHub mirror, Nightly metadata, Mojang version list and pinned 26.3
manifest all returned HTTP 200 during this investigation. HEAD checks also returned 200 for
the Nightly Windows archive/checksums/signature, both pinned JAR URLs, the asset index,
a sample Maven library, and the Java runtime index. These were availability probes, not
an end-to-end authenticated Minecraft installation. The original users' exact 503/504 URLs
are not available, so that outage was not reproduced or attributed to an invented endpoint.

GETs now have at most four attempts, exponential backoff with bounded jitter, and `Retry-After`
seconds/HTTP-date handling. A delay over 30 seconds is reported rather than retried prematurely.
Permanent failures are not retried. Redundant sound-download retry loops were removed so retries
do not multiply. Proxy selection and GitHub credential scoping remain in place. OAuth POSTs are
not automatically replayed.

Interrupted archives resume within the current attempt only with a strong ETag and validated
Content-Range. A server ignoring Range restarts from zero. Temporary files are removed after
failure, and only complete files passing the expected hash replace the destination. Whole-body
manifest/library reads retry interruptions. Existing valid cached Minecraft files remain reusable;
large-file partial resume across launcher restarts is not implemented. Server JAR downloads and
previously cached server JARs are checked against their manifest SHA-1. A stale/corrupt cached JAR
is replaced only after its replacement verifies successfully.

## macOS and CI

The failing `an_app_bundle_updates_in_place` fixture copied and re-signed Apple's `/usr/bin/true`.
It now compiles a minimal native Rust executable and still checks permissions, signing, ZIP
extraction, bundle replacement, execution, strict signature verification and tamper rejection.
The old symptom was an unsuccessful child process at the execution assertion, rather than a
permission error reported while starting the child. The regression test remains enabled. Its precise kernel
termination reason cannot be established on this Windows host; the new fixture removes the
system-binary signing assumption and reports exit status/mode on failure. Native ARM64/x64 runs
are still required before calling the macOS issue verified.

The launcher workflow now uses GitHub-hosted `macos-15` and `macos-15-intel`; the Elyhorizon
workflow already had both native architectures. See [GitHub runner documentation](https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/choose-the-runner-for-a-job).
Runtime Metal shader features, Windows static CRT, Linux packaging, signatures/integrity checks,
Microsoft public configuration and existing public Ely Actions Variables are retained.

All Elyhorizon package jobs run offline validator tests before live OAuth preflight. Installed
Windows programs, Linux archive/AppImage programs and copied macOS bundles then run
`tools/check_launcher_package.py`. It creates fresh homes and cwd, removes inherited developer
configuration, checks bundled S256 configuration and empty accounts across separate processes,
and starts the bundled game's help command. This is an offline package smoke check, not proof
of successful browser login or gameplay. No workflows were triggered here.

## Safe manual verification checklist

Use a test account and fresh disposable folders, never delete or rename the existing launcher
profile. `tools/check_launcher_package.py --launcher PATH --client PATH --client-id elyhorizondesktop`
performs the offline checks automatically. For interactive testing, create fresh directories for
`HORIZON_HOME`, `HORIZON_LAUNCHER_HOME`, `XDG_DATA_HOME`, `XDG_CONFIG_HOME`,
`HORIZON_LAUNCHER_TRASH`, `HORIZON_LAUNCHER_CLIPBOARD`, and `HORIZON_LAUNCHER_DESKTOP`;
set `HORIZON_LAUNCHER_NO_SELF_UPDATE=1`, `HORIZON_NO_AUDIO=1`, and telemetry off.
Launch from the fresh working directory. Direct game invocations also need `--game-dir` pointing
there. Remove runtime `HORIZON_ELY_*` variables so the packaged public ID is tested.

| Scenario | Required observation |
| --- | --- |
| Fresh installation / empty launcher profile | No remembered accounts; embedded public ID works without environment overrides |
| Fresh Ely browser sign-in | S256, exact issuer/state, loopback callback, real profile and successful secure save |
| Invalid state, issuer, Host, duplicate or ambiguous callback | Login is not accepted; existing selected account stays; no activation |
| Denial, cancellation and five-minute timeout | Clear local error; no credentials or account switch |
| Browser return while launcher is minimized | Existing window restores once; denied focus produces platform attention where supported |
| Restart after successful login | Same selected account; no need to enter developer environment variables |
| Expired/revoked token | Correct refresh or explicit reauthentication; Microsoft is not requested for an Ely account |
| Missing public build ID | Offline configuration test fails; live preflight prevents packaging |
| Empty Minecraft cache | Authenticated Ely installation downloads and verifies official files, then launches with the real nick/UUID |
| Simulated 503/504 / timeout | Bounded retries, correct error and sanitized endpoint; no broken installation promoted |
| Interrupted or ignored Range response | Correct resume/restart, final integrity check, no leftover partial treated as installed |
| Permanent HTTP failure / corrupt archive | Visible failure; an existing valid destination is preserved |
| Microsoft account | Existing ownership, refresh, switching and launch flow still work with its public build ID |
| Older settings / accounts | Accounts/settings persist; Linux fallback migrates without losing sessions |
| macOS ARM64 and Intel updates | Native bundle test passes, updated app runs and strict signature remains valid |
| Windows clean VM | Installer and both programs run without installed developer tools/VC redistributable |
| Linux Ubuntu / Arch, X11 / Wayland | Package startup, native graphics and compositor-limited focus verified separately |
| macOS copied app / DMG | Correct architecture, Metal shaders, browser return and Gatekeeper behavior verified natively |

## Verification record

The final Rust checks passed on Windows x64:

- `cargo fmt --manifest-path launcher/Cargo.toml --check`.
- `cargo clippy --manifest-path launcher/Cargo.toml --all-targets --locked -- -D warnings`.
- `cargo test --manifest-path launcher/Cargo.toml --locked --lib --bins`: **544 passed**, 7 existing ignored tests.
- The same tests with `--release` and `RUSTFLAGS=-C target-feature=+crt-static`: **544 passed**, 7 existing ignored tests.
- The final release launcher EXE built with that same static CRT setting.
- All existing workflows passed `actionlint`; the public-tree and whitespace checks passed.

The actual local Inno installer was compiled and installed with an isolated AppId and target folder.
Both installed program hashes matched the staged programs. `tools/check_imports.py` accepted their
system-only DLL imports. The installed EXE passed `tools/check_launcher_package.py` with fresh
homes, empty accounts across separate processes and no runtime authentication settings; the game's
help command also started. Uninstallation removed the test programs and preserved a player-data
marker outside their installation directory. No existing installation was replaced.

The game EXE in this local installer was the checksum-verified, **unchanged** game from the public
0.2.2 Windows ZIP, not a newly compiled game. Its source areas have no differences between the
0.2.2 tag and the initial `main`; this task changes launcher code only. The local installer is an
offline review artifact, not a release candidate that passed the live OAuth preflight. The normal
workflow will currently stop at that preflight.

The complete Python suite on the final repository state passed: **621 tests, 35 skipped**
(`python -m unittest discover -s tests`). The skip count reflects existing platform/fixture
requirements; no failing tests were disabled. The successful full run followed fixes for the
new script's UTF-8 BOM, the common game environment helper, and interpreter discovery.
A Windows Python interpreter-discovery failure found by the full suite was fixed by preferring the
running interpreter and ignoring non-executable Store aliases. The package checker now shares
`lab.game_env` with the other harnesses. Initial failures were fixed and rerun, not ignored.

A pristine Windows VM, native Linux/macOS runs, real Microsoft login and a fresh end-to-end login
using `elyhorizondesktop` have not been verified. The last one is currently blocked by the provider's
HTTP 401 `invalid_client` response, not by a missing client secret. Do not publish or describe
this release as fully verified until those checks have been completed.

Windows visual/focus inspection was unavailable because the Computer Use helper returned a
missing-native-pipe error after retries/reset. Compilation and the once-only completion transition
are checked; actual minimized-window/browser interaction still needs the manual platform test.
macOS distribution is still ad hoc signed rather than Developer ID notarized, as disclosed in the
existing packaging guide. No Gatekeeper or OS focus restrictions were disabled.

## Changed files

These paths are relative to the starting `main` commit and include the externally saved interim
commits plus the local follow-up fixes:

- `.github/actionlint.yaml`
- `.github/workflows/elyhorizon-release.yml`
- `.github/workflows/launcher.yml`
- `README.md`
- `README.ru.md`
- `docs/elyby-account.md`
- `docs/elyhorizon-0.2.2-verification.md`
- `docs/elyhorizon-releases.md`
- `docs/microsoft-account.md`
- `launcher/Cargo.toml`
- `launcher/lang/de.json`
- `launcher/lang/es.json`
- `launcher/lang/fr.json`
- `launcher/lang/ja.json`
- `launcher/lang/pl.json`
- `launcher/lang/pt-br.json`
- `launcher/lang/ru.json`
- `launcher/lang/tr.json`
- `launcher/lang/uk.json`
- `launcher/lang/zh.json`
- `launcher/src/account.rs`
- `launcher/src/account/ely.rs`
- `launcher/src/account/ely/oauth.rs`
- `launcher/src/account/tests.rs`
- `launcher/src/backend.rs`
- `launcher/src/github.rs`
- `launcher/src/lib.rs`
- `launcher/src/login_focus.rs`
- `launcher/src/main.rs`
- `launcher/src/mojang.rs`
- `launcher/src/mojang_tests.rs`
- `launcher/src/network.rs`
- `launcher/src/network/tests.rs`
- `launcher/src/while_playing.rs`
- `tests/test_ely_oauth_release.py`
- `tests/test_launcher_package.py`
- `tools/check_ely_oauth.py`
- `tools/check_launcher_package.py`
- `tools/port_regen.py`

## Final outcome

The local code and Windows offline packaging checks are ready for review. The release authentication
goal is **not claimed achieved**: the exact requested public Client ID is still rejected by Ely.by
with HTTP 401 `invalid_client`, including with redirect port 38467. Resolve that registration/ID
mismatch against the existing Actions Variable, then repeat the live preflight and real browser
sign-in on the actual CI artifacts. No hosted backend or confidential client secret is needed.
Native macOS/Linux execution, graphical focus behavior, real Microsoft login, a clean Windows VM,
and an authenticated empty-cache Minecraft installation remain the explicit manual/CI checks.
