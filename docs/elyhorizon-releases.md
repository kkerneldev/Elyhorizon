# Elyhorizon releases on GitHub

`.github/workflows/elyhorizon-release.yml` builds Windows x64 on GitHub-hosted
runners. It does not require the upstream project's self-hosted runners, signing
environment, or a personal access token. The release job uses GitHub's automatic
`GITHUB_TOKEN` with `contents: write`; the build job has read access only.

## Publish from Actions

1. Push the workflow and source changes to your repository's default branch.
2. Open **Actions → Elyhorizon Windows release → Run workflow**.
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
with the Windows C runtime linked in, checks their DLL imports, and uploads:

- `Elyhorizon-<version>-windows-x64-setup.exe` (installer)
- `Elyhorizon-<version>-windows-x64.zip` (portable alternative)
- `SHA256SUMS`

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

Ely.by application settings saved in one computer's credential manager do not
travel with the release. Configure browser sign-in on the destination computer
using [the Ely.by setup instructions](elyby-account.md). The client secret must
not be embedded through Actions secrets, build environment variables or archives.
