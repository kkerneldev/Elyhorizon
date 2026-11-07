# Ely.by accounts

Settings → Account → **Sign in with Ely.by** opens the system browser. The launcher
uses Authorization Code with **S256 PKCE**, a random state, and an ephemeral
`http://127.0.0.1:<port>/oauth/ely` callback. It checks the exact callback path,
state and `iss=https://account.ely.by`, then sends the code and the private verifier
directly to Ely.by over HTTPS. Codes, verifiers and bearer tokens never follow
HTTP redirects. No hosted backend or client secret is used.

Credentials and two-factor codes stay on Ely.by. User access/refresh tokens are
kept by the existing account secret store. Cancelling, timeout and rejected
callbacks do not add or select an account. Microsoft sign-in is unchanged.

## Release maintainer: register a public Desktop application

1. Open https://account.ely.by/dev and create an application of type **Desktop application**.
2. Copy its public client ID. A Website registration such as the old `elyhorizon2`
   is confidential and still requires a secret; omitting that secret does not turn
   it into a public client. Create a Desktop registration instead.
3. In your GitHub repository, open **Settings → Secrets and variables → Actions → Variables**.
   Add `HORIZON_ELY_CLIENT_ID` with that public Desktop ID.
4. Run the Elyhorizon release workflow. It validates the live registration before
   building, compiles the public ID into the launcher and tests the installed EXE
   after removing the build environment and using an empty launcher home.

Only this public ID is distributed. **Do not set or embed HORIZON_ELY_CLIENT_SECRET**;
this launcher no longer reads it. No Ely.by secret belongs in Actions Secrets,
source, installers or release artifacts. Rotate previously disclosed Website
credentials in Ely.by and use the public Desktop registration for new releases.

Users of a correctly built release simply click Sign in with Ely.by; no developer
keychain, environment variables or application setup is required on their computer.

For source development, `HORIZON_ELY_CLIENT_ID` can be supplied when running Cargo
and will be compiled into that build. A runtime override takes precedence. The
optional `account ely-configure` command accepts public `client_id` and optional
loopback `redirect_uri` on stdin, refuses `client_secret`, and is not part of the
normal installed-user flow. Its port is replaced by an OS-assigned free port.
A legacy local application record is not needed by a configured release.

`horizon-launcher --json account ely-status` prints only the effective and bundled
public ID, PKCE method and flow. A missing ID reports a release configuration error;
`invalid_client` reports that the maintainer must use a public Desktop registration,
rather than asking the player for a secret. Provider error descriptions are never
reflected into diagnostics. A refresh failure preserves the existing selected
account; an invalid/expired grant asks the player to sign in again.

## Provider support and verification

The older [OAuth guide](https://docs.ely.by/en/oauth.html) describes Website clients
and is incomplete for Desktop apps. The live
[discovery document](https://account.ely.by/.well-known/openid-configuration)
advertises S256 and token authentication method `none`.

We inspected the official Apache-2.0 Ely.by implementations as protocol references,
not as copied implementation code:

- [accounts at d4b7f3e](https://github.com/elyby/accounts/tree/d4b7f3eff3572de5f917968321ddc6c007474da0): public Desktop client authentication, loopback redirects, discovery and PKCE tests.
- [accounts-frontend at 7ab69d0c](https://github.com/elyby/accounts-frontend/tree/7ab69d0c7d258538f8560df7ad209fe8becd0620): forwarding authorization parameters, including PKCE, through browser validation/completion.
- [RFC 7636](https://www.rfc-editor.org/rfc/rfc7636): S256 algorithm and independent test vector.

`python tools/check_ely_oauth.py` checks discovery, rejects a registration that does
not require PKCE, verifies S256/loopback authorization validation, checks that `plain`
is rejected, and submits an intentionally invalid code to check secret-free client
authentication. It never requests or receives a real user token. This preflight is
**not an interactive end-to-end sign-in test**. Before calling a release verified,
install its actual setup EXE on clean Windows, complete browser consent, check the
selected profile, restart the launcher and verify refresh. The maintainer must
perform that check with their registered Desktop ID and a real test account.

### Local verification on 2026-11-07

With public Desktop ID `elyherizondesktop`, the live preflight passed. A locally
compiled installer containing the rebuilt launcher and Horizon client was installed
under an isolated test application ID and fresh data homes. With all runtime
`HORIZON_ELY_*` variables removed, the installed EXE reported the embedded public
ID, completed real browser sign-in and the loopback callback, selected the profile,
and refreshed it from a separate process to `Ready for online play`. Both installed
program hashes matched the staged files. Uninstallation preserved the isolated
profile and player-data marker.

This used a local development launcher build and an optimized `fast` game build,
not a GitHub-produced release or a pristine Windows VM. The same checks are wired
into the release workflow, but its actual artifact still needs the clean-Windows
interactive verification above. No backend deployment is required.

## Automatic installation and file sources

A freshly authenticated Ely.by account can install missing Minecraft files,
libraries, assets and Java through the existing downloader. Cached files are
checked against their published size and SHA-1; incomplete downloads resume.
The offline developer option and an unverified cached session do not authorize
a download. Microsoft accounts still use the existing ownership check.

Ely.by handles identity and textures. Minecraft files come from the official
URLs in the version manifest; Java comes from the configured official runtime
sources. The publicly documented Ely API does not expose a complete Minecraft
distribution mirror. The launcher does not label Mojang downloads as Ely.by
downloads or invent replacement CDN endpoints.

Java instances automatically use authlib-injector 1.2.8, official build 56. It
is downloaded into the launcher's data directory, verified against its pinned
SHA-256 on every use, and activated only for Ely sessions.

For Horizon, distribute the updated `horizon-client.exe` beside
`horizon-launcher.exe`. The launcher selects this bundled client when the
installed version's Minecraft version and client/server hashes match its pins.
Imported and native-mod builds retain their own executable. An upstream client
without Ely support cannot read an Ely session. Updated Horizon refuses a bad
session handoff instead of silently starting with a generated offline name.

To build and publish the Windows launcher and matching client from your fork,
use the [Elyhorizon GitHub Actions release workflow](elyhorizon-releases.md).

## Accounts, servers and CLI

The account switcher supports Microsoft and Ely.by together. `accounts.json`
uses `ely:<undashed UUID>` for an Ely launcher's identity; games receive the real
UUID. The `owner` flag continues to mean Microsoft ownership, not Ely sign-in.
Old Ely Yggdrasil sessions remain refreshable; re-signing uses browser OAuth.

`horizon-launcher account ely-sign-in` prints the authorization URL and waits
for the browser. Existing `account select`, `refresh`, `session` and `sign-out`
commands accept the namespaced account ID. CLI/MCP account lists expose only
public provider, UUID and authentication status, never tokens.

Sign out removes local credentials even while offline. Ely does not document
OAuth token revocation; revoke the app in your Ely account's applications page
to end its remote access. Legacy Yggdrasil sessions use `/auth/invalidate`.

Ely accounts join servers configured for Ely.by, rather than Microsoft-only
online servers. For a Horizon server set `HORIZON_AUTH_PROVIDER=elyby`, keep
`online-mode=true`, and set `enforce-secure-profile=false`: Ely does not issue
Mojang secure-chat certificates. Authentication never falls back between
identity providers.

Protocol references (documentation only):

- https://docs.ely.by/en/oauth.html
- https://docs.ely.by/en/minecraft-auth.html
- https://docs.ely.by/en/skins-system.html
- https://docs.ely.by/en/authlib-injector.html
