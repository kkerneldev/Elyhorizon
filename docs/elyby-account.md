# Ely.by accounts

Settings → Account → **Sign in with Ely.by** opens the system browser. Enter
your credentials and two-factor code only on `account.ely.by`. The launcher
receives a one-use authorization code through a loopback callback, checks its
random `state`, exchanges it for a session, and stores tokens in the existing
OS secret store. It has no password or TOTP fields. Cancel and a five-minute
timeout close the callback listener; a rejected or cancelled flow never adds
an account. Sign in again restores the selected identity without switching to
a different account.

## Configure the launcher application once

Ely.by requires a registered OAuth application. Registration cannot be replaced
with a made-up client ID or another launcher's credentials.

1. Open https://account.ely.by/dev under the account that owns Elyhorizon.
2. Register an application named **Elyhorizon**, type **Website**, with redirect
   URI `http://127.0.0.1:38467/oauth/ely`.
3. Provide `HORIZON_ELY_CLIENT_ID` and `HORIZON_ELY_CLIENT_SECRET` to the launcher
   process. Do not commit the secret or send it in chat. The launcher does not
   put it in authorization URLs, account-list output, or session files.
4. If another callback address was registered, set `HORIZON_ELY_REDIRECT_URI`
   to that exact URI. Only HTTP on `127.0.0.1`, an explicit nonzero port and a
   callback path are accepted. Remote callbacks and wildcard binds are refused.

Alternatively, run `horizon-launcher account ely-configure` once and provide a
JSON line on stdin with `client_id`, `client_secret`, and optional `redirect_uri`.
Use a secure prompt or credential manager to supply it, not shell arguments or
history. The app configuration is saved as a separate OS keychain entry and is
loaded automatically on later launches. Environment variables override it.
This configuration command is intentionally not exposed as an MCP tool and
prints only the public client ID, never the secret.

Requested scopes are `account_info offline_access minecraft_server_session`.
The documented Ely OAuth protocol requires `client_secret` during exchange and
refresh and does not document PKCE. For a public distributed application, keep
confidential app credentials in a managed exchange service rather than embedding
them in an executable. This local integration reads credentials at runtime and
does not provision or host such a service.

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
