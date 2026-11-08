# Elyhorizon releases on GitHub

`.github/workflows/elyhorizon-release.yml` builds Windows x64, Linux x86_64 and macOS arm64/x64 on GitHub-hosted
runners. It does not require the upstream project's self-hosted runners, signing
environment, or a personal access token. The release job uses GitHub's automatic
`GITHUB_TOKEN` with `contents: write`; the build jobs have read access only.

## Publish from Actions

1. Push the workflow and source changes to your repository's default branch.
2. Open **Actions → Elyhorizon release → Run workflow**.
3. Choose the branch and enter a new version, for example `0.2.1`.
4. Leave **publish** enabled to create a release, or disable it to build only.

Alternatively, push a tag such as `elyhorizon-v0.2.1`. That automatically builds
and publishes that commit. Versions containing a suffix, such as `0.2.1-beta.1`,
become prereleases. Releases are not marked as the repository's latest, so they
do not replace the upstream game channel. Existing tags must point to the exact
commit being built; existing releases are not overwritten. If publishing fails
after a draft is created, complete that draft from GitHub's Releases page.

If Actions are disabled on the fork, enable them in the Actions tab first.
Repository or organization policies must permit workflows to write release
contents. No additional PAT or Ely.by secret needs to be added for this workflow.
These launchers do not offer automatic replacement from the upstream launcher
feed. Run newer Elyhorizon installers from this repository's Releases page.

## Download

The workflow runs launcher and authentication tests, builds both executables
for each platform (static Windows C runtime, checked DLL imports) and uploads:

- `Elyhorizon-<version>-windows-x64-setup.exe` (installer)
- `Elyhorizon-<version>-windows-x64.zip` (portable alternative)
- `Elyhorizon-<version>-linux-x86_64.AppImage`
- `Elyhorizon-<version>-linux-x86_64.tar.gz`
- `Elyhorizon-<version>-macos-arm64.dmg` and `.zip`
- `Elyhorizon-<version>-macos-x64.dmg` and `.zip`
- `SHA256SUMS` (all platforms)

The installer installs both the launcher and matching Horizon client under
`%LOCALAPPDATA%\Programs\Elyhorizon`, creates a Start menu shortcut and optionally
a desktop shortcut. It has English and Russian setup languages, needs no admin
access, and provides an entry in Windows Installed Apps for uninstallation.
Updates use the same application ID and install location. Uninstallation removes
installed program files and shortcuts; saved worlds and account data are kept.
The workflow compiles it with the GitHub runner's Inno Setup 6, then installs and
uninstalls it in a temporary folder and checks both executable hashes before publishing.

The ZIP contains the launcher, matching Horizon client, license notices and setup
instructions. Keep both executables together to use Ely.by support in Horizon.
Build-only runs expose the same files in the `elyhorizon-windows-x64` artifact.
Minecraft jars, assets, prepared resources and OAuth secrets are not packaged.

## Linux distribution compatibility

The Linux job uses GitHub-hosted Ubuntu 22.04, targeting glibc 2.35 or newer.
The AppImage is intended for Ubuntu 22.04+, Debian 12+, current Arch and compatible
Fedora desktop systems. This is a compatibility target, not a claim that every
one of those distributions has been manually tested. Musl-only systems need a
separate build. Graphics drivers, glibc and session/device libraries stay supplied
by the distribution; other required application libraries are bundled.

Both binaries are shipped inside the same AppImage, so the launcher can find its
matching client. The workflow tests the archive after extraction, checks embedded
Ely.by configuration in both packages without runtime variables, and compares the
AppImage's client with the original after applying the same ELF run path change.
The AppImage can use `--appimage-extract-and-run` without FUSE. Linux artifacts are
uploaded as `elyhorizon-linux-x86_64`; publication waits for all platform jobs and
merges their checksums into one `SHA256SUMS` file.

A second smoke check runs the actual AppImage in a fresh `archlinux:base`
container, checking launcher startup, embedded public authentication configuration
and the bundled client's help command. This covers startup on rolling Arch as well
as the Ubuntu build host; it does not exercise rendering or interactive browser
authentication. Arch's container image and package repositories track current Arch.

## macOS packaging and verification

Two native GitHub-hosted runners build the Mac packages: `macos-15` for Apple
Silicon and `macos-15-intel` for Intel. No upstream self-hosted runner or Rosetta
is required. The deployment target and bundle minimum are macOS 12.0; the actual
startup checks run on macOS 15. Older OS and graphical compatibility need manual
verification. GPUI uses runtime Metal shaders, so a separate Metal compiler
download is not required.

`packaging/macos/elyhorizon.sh` checks both Mach-O architectures, creates
`Elyhorizon.app` with the launcher and matching client in `Contents/MacOS`, adds
licenses and the icon, then signs nested binaries and the bundle ad hoc. It emits
a ZIP and a DMG with an Applications link. The workflow mounts the DMG read-only,
copies the app into a temporary installation folder, verifies signatures and
executable contents, and checks embedded PKCE configuration without runtime Ely
variables. Publication waits for both Mac architectures, Windows and Linux.

These packages are **not Apple-notarized** and do not use a Developer ID certificate.
Gatekeeper may require explicit user approval; this workflow does not disable it.
Developer ID signing and notarization require a separate Apple developer setup.
See [Apple's distribution guidance](https://developer.apple.com/macos/distribution).

## Ely.by release configuration

Create an Ely.by **Desktop application** and set repository Actions Variable
`HORIZON_ELY_CLIENT_ID` to its public ID. Do not use a Website registration, and do
not supply a client secret. All platform jobs compile this safe public value
into the executable. `HORIZON_MSA_CLIENT_ID` remains a separate public variable for
Microsoft sign-in and is passed through unchanged.

The project Desktop ID verified on 2026-11-07 is `elyherizondesktop`. Set that value
in Actions Variables for this repository; other forks should register their own app.

Before publishing, the workflow runs `tools/check_ely_oauth.py`. An empty ID, a
confidential Website client, unavailable discovery, or rejected PKCE causes a clear
failure; such builds must not be published as working releases. The installer smoke
test invokes `account ely-status` from the installed EXE with no runtime Ely.by
variables and a fresh launcher home, verifying its embedded public ID and PKCE mode.

This verifies registration and packaged configuration, not human browser consent.
Complete the manual clean-Windows sign-in and refresh checklist in
[the Ely.by guide](elyby-account.md#provider-support-and-verification) before marking
a release's authentication end-to-end verified. No separately hosted backend is needed.
