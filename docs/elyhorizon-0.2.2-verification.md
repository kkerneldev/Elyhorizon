# Elyhorizon 0.2.2 local stabilization and verification

All changes are on local `main`, starting at `a7d5f71bab`. At the initial check, both
`origin/main` and the live remote `refs/heads/main` had that same commit. The previous
`sync/upstream-2026-11-08` branch remains separate; it was not merged into `main`.
No commits, pushes, tags, PRs, workflow dispatches or releases are part of this work.

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
simulated keyboard input. macOS/Linux request activation and attention through their window
backend. Wayland/compositor rules can refuse focus; forced focus is not promised. Native GUI
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
manifest all returned HTTP 200 during this investigation. The original users' exact 503/504 URLs
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
large-file partial resume across launcher restarts is not implemented. Server JAR downloads now
also verify their manifest SHA-1 before promotion.

## macOS and CI

The failing `an_app_bundle_updates_in_place` fixture copied and re-signed Apple's `/usr/bin/true`.
It now compiles a minimal native Rust executable and still checks permissions, signing, ZIP
extraction, bundle replacement, execution, strict signature verification and tamper rejection.
The old symptom was an unsuccessful child process, not an ignored test. Its precise kernel
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

Final local build/test results are recorded after the active checks complete. A pristine Windows
VM, native Linux/macOS runs, real Microsoft login and a fresh end-to-end login using
`elyhorizondesktop` have not been verified. The last one is currently blocked by the provider's
HTTP 401 `invalid_client` response, not by a missing client secret. Do not publish or describe
this release as fully verified until those checks have been completed.
