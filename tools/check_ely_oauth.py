#!/usr/bin/env python3
"""Release gate for Ely.by public Desktop registration; never uses a secret or user token."""
import argparse
import base64
import hashlib
import json
import os
import secrets
import sys
import urllib.error
import urllib.parse
import urllib.request

ISSUER = "https://account.ely.by"
SCOPES = "account_info offline_access minecraft_server_session"


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def request(path, fields=None, post=False):
    encoded = urllib.parse.urlencode(fields or {}).encode()
    url = ISSUER + path
    if not post and encoded:
        url += "?" + encoded.decode()
    req = urllib.request.Request(url, data=encoded if post else None,
                                 headers={"Accept": "application/json"})
    try:
        response = urllib.request.build_opener(NoRedirect).open(req, timeout=20)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        return response.code, json.loads(response.read(65536))


def check(client_id, send=request):
    if not client_id or not client_id.strip():
        raise ValueError("Set repository Actions Variable HORIZON_ELY_CLIENT_ID to your public Desktop application ID. No CLIENT_SECRET is needed.")
    status, metadata = send("/.well-known/openid-configuration")
    if (status != 200 or metadata.get("issuer") != ISSUER
            or "S256" not in metadata.get("code_challenge_methods_supported", [])
            or "none" not in metadata.get("token_endpoint_auth_methods_supported", [])):
        raise ValueError("Ely.by does not advertise the required public S256 PKCE flow; refusing to publish.")
    verifier = secrets.token_urlsafe(32)
    challenge = base64.urlsafe_b64encode(hashlib.sha256(verifier.encode()).digest()).rstrip(b"=").decode()
    fields = dict(client_id=client_id.strip(), redirect_uri="http://127.0.0.1:43210/oauth/ely",
                  response_type="code", scope=SCOPES, state=secrets.token_urlsafe(32))
    status, value = send("/api/oauth2/v1/validate", fields)
    if value.get("error") == "invalid_client":
        raise ValueError(f"Ely.by returned HTTP {status} invalid_client for the configured public Client ID {client_id.strip()!r}. Verify this exact ID in the Ely.by Desktop registration; no client secret will be used.")
    if status != 400 or value.get("parameter") != "code_challenge":
        raise ValueError("Ely.by did not require PKCE for this ID. Register a Desktop application, not the old Website application (elyhorizon2).")
    pkce = dict(fields, code_challenge=challenge, code_challenge_method="S256")
    status, value = send("/api/oauth2/v1/validate", pkce)
    if status != 200 or value.get("success") is not True:
        raise ValueError("Ely.by rejected public Desktop authorization with S256 and a loopback redirect. Check the registered client ID.")
    status, value = send("/api/oauth2/v1/validate", dict(pkce, code_challenge_method="plain"))
    if status != 400 or value.get("parameter") != "code_challenge_method":
        raise ValueError("Ely.by did not reject PKCE downgrade; refusing to publish.")
    # An intentionally invalid code cannot mint a token. This checks client authentication only.
    status, value = send("/api/oauth2/v1/token", dict(
        client_id=fields["client_id"], redirect_uri=fields["redirect_uri"],
        grant_type="authorization_code", code="elyhorizon-release-probe-" + secrets.token_urlsafe(32),
        code_verifier=verifier), True)
    if status != 400 or value.get("error") != "invalid_grant":
        raise ValueError("Secret-free code exchange was not accepted for this application. A public Desktop registration is required.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--client-id", default=os.environ.get("HORIZON_ELY_CLIENT_ID"))
    args = parser.parse_args()
    try:
        check(args.client_id)
    except ValueError as error:
        print(f"Ely.by release configuration: {error}", file=sys.stderr)
        return 1
    except Exception:
        # No response bodies or request URLs: provider errors can echo credentials or codes.
        print("Could not verify Ely.by public-client configuration. Check network/service availability and retry.", file=sys.stderr)
        return 1
    print("Ely.by public Desktop registration verified: S256, loopback redirect, no client secret. Interactive user sign-in is still required for end-to-end verification.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
