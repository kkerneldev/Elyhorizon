#!/usr/bin/env python3
"""Offline smoke check of installed programs, using only fresh disposable player homes."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

HOMES = ("HORIZON_HOME", "HORIZON_LAUNCHER_HOME", "XDG_DATA_HOME", "XDG_CONFIG_HOME",
         "HORIZON_LAUNCHER_TRASH", "HORIZON_LAUNCHER_CLIPBOARD", "HORIZON_LAUNCHER_DESKTOP")


def check(launcher, client, client_id, appimage=False, run=subprocess.run):
    launcher, client = Path(launcher).resolve(), Path(client).resolve()
    if not client_id or not client_id.strip():
        raise ValueError("Expected public Client ID is missing")
    with tempfile.TemporaryDirectory(prefix="elyhorizon-package-") as scratch:
        env = dict(os.environ)
        for key in list(env):
            if key.startswith("HORIZON_"):
                env.pop(key)
        for key in HOMES:
            directory = Path(scratch, key)
            directory.mkdir()
            env[key] = str(directory)
        env.update(HORIZON_LAUNCHER_NO_SELF_UPDATE="1", HORIZON_NO_AUDIO="1", HORIZON_TELEMETRY_URL="off")
        prefix = [str(launcher)] + (["--appimage-extract-and-run"] if appimage else [])
        def call(args):
            result = run(args, cwd=scratch, env=env, capture_output=True, text=True, timeout=60, check=False)
            if result.returncode:
                # Do not echo arbitrary stderr, which might contain provider data.
                raise ValueError(f"Packaged program failed its offline smoke check (exit {result.returncode})")
            return result.stdout
        config = json.loads(call(prefix + ["--json", "account", "ely-status"]))
        if not (config.get("client_id") == client_id and config.get("bundled_client_id") == client_id
                and config.get("flow") == "authorization_code_pkce" and config.get("code_challenge_method") == "S256"
                and config.get("client_secret_required") is False):
            raise ValueError("Installed launcher lacks the expected bundled public S256 PKCE configuration")
        for _ in range(2):
            accounts = json.loads(call(prefix + ["--json", "account", "list"]))
            if accounts.get("accounts") != [] or accounts.get("can_play") is not False:
                raise ValueError("Fresh launcher home unexpectedly contains accounts or remembered authorization")
        if not call(prefix + ["--version"]).startswith("horizon-launcher "):
            raise ValueError("Packaged launcher returned no version")
        call([str(client), "--game-dir", env["HORIZON_HOME"], "--help"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--launcher", required=True)
    parser.add_argument("--client", required=True)
    parser.add_argument("--client-id", required=True)
    parser.add_argument("--appimage", action="store_true")
    args = parser.parse_args()
    try:
        check(args.launcher, args.client, args.client_id, args.appimage)
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"Package verification failed: {error}")
        return 1
    print("Installed programs passed isolated offline configuration/startup checks; interactive login and gameplay are not verified by this check.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
