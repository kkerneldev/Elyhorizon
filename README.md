<p align="center"><strong>English</strong> · <a href="README.ru.md">Русский</a></p>

<p align="center">
  <a href="https://github.com/horizon-rust/horizon"><img src="docs/media/brand/icon.svg" alt="Horizon logo" width="104" height="104"></a>
  &nbsp;&nbsp;&nbsp;&nbsp;
  <a href="https://ely.by"><img src="docs/media/elyhorizon/elyby-logo.png" alt="Ely.by logo" width="104" height="104"></a>
</p>

<h1 align="center">Elyhorizon</h1>

<p align="center">
  <b>Horizon with Ely.by accounts and its own Windows installer.</b><br>
  An independent fork of <a href="https://github.com/horizon-rust/horizon">Horizon</a>, a Minecraft: Java Edition 26.3 client written in Rust.
</p>

<p align="center">
  <a href="https://github.com/kkerneldev/Elyhorizon/actions/workflows/elyhorizon-release.yml"><img src="https://github.com/kkerneldev/Elyhorizon/actions/workflows/elyhorizon-release.yml/badge.svg" alt="Elyhorizon build"></a>
  <img src="https://img.shields.io/badge/Windows-x64-5b3fd6" alt="Windows x64">
  <img src="https://img.shields.io/badge/Ely.by-Browser%20OAuth-208260" alt="Ely.by sign-in through your browser">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-5b3fd6" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://github.com/kkerneldev/Elyhorizon/releases"><b>Download installer</b></a> ·
  <a href="docs/elyby-account.md"><b>Ely.by setup</b></a> ·
  <a href="docs/elyhorizon-releases.md"><b>Build a release</b></a> ·
  <a href="https://github.com/kkerneldev/Elyhorizon/issues"><b>Report an issue</b></a> ·
  <a href="https://github.com/horizon-rust/horizon"><b>Original Horizon</b></a>
</p>

## About this fork

**Elyhorizon is a separate Horizon distribution with Ely.by integration.** It is
built on the original Horizon Rust client. This fork adds browser sign-in with
Ely.by, passes your account name and UUID to the game, loads skins, and provides
a Windows installer that installs the launcher alongside its matching client.

This project is developed independently of the Horizon and Ely.by teams. The
logos identify the projects behind the integration and do not imply an official
partnership.

| Feature | What Elyhorizon provides |
| --- | --- |
| Ely.by accounts | Sign in through your system browser, select an account, and refresh sessions |
| Player name and skin | Pass the selected account to Horizon and load textures from Ely.by |
| Minecraft Java | Use authlib-injector for Java instances launched with Ely.by |
| Installation | One installer for the launcher and Horizon Rust client |
| Portable launch | A ZIP containing both executables, with no installation required |
| Release builds | GitHub Actions builds Windows x64, tests installation, and publishes the files |

## Download and install

1. Open [this fork's Releases](https://github.com/kkerneldev/Elyhorizon/releases).
2. Download **`Elyhorizon-<version>-windows-x64-setup.exe`**.
3. Run the installer, choose a folder, and optionally add a desktop shortcut.
4. Open **Elyhorizon** from the Start menu, add your Ely.by account, and select it before playing.

The installer runs for the current user without administrator access. It places
both executables together and adds an uninstall entry to Windows Installed Apps.
You need Windows x64 and a GPU suitable for Horizon.

For a portable setup, download `Elyhorizon-<version>-windows-x64.zip`, extract
**the entire folder**, and run `horizon-launcher.exe`. Keep `horizon-launcher.exe`
and `horizon-client.exe` together so the launcher can use the client with Ely.by
support. Imported builds and builds with native mods use their own executable.

If no downloads are available yet, run the
[release workflow](docs/elyhorizon-releases.md) in your GitHub repository first.

## Sign in with Ely.by

Authentication opens the Ely.by website in your browser. Enter your password and
two-factor code on the website; tokens are saved in your system credential store.

Browser sign-in requires a one-time OAuth application configuration on the
computer running the launcher. The application secret is not included in the
release executable and does not transfer automatically from another computer.
See [account setup and commands](docs/elyby-account.md).

Ely.by provides accounts and textures. Minecraft files, libraries, and resources
are downloaded from official sources or reused from an existing installation.
They are not included in the Elyhorizon installer.

Ely.by accounts work with servers configured for Ely.by authentication.
Access to servers using Microsoft authentication depends on that server's requirements.

## A look at Horizon

These upstream Horizon screenshots show the base interface and game on which
this fork is built.

<p align="center">
  <img src="docs/media/launcher-play.webp" alt="Horizon launcher home page" width="780">
</p>

<p align="center">
  <img src="docs/media/sunset.webp" alt="Sunset over a village in the Horizon Rust client" width="780">
</p>

## Build a release with GitHub Actions

Open **Actions → Elyhorizon Windows release → Run workflow**, enter a new
version such as `0.2.1`, and enable `publish`. You can also push a tag such as
`elyhorizon-v0.2.1`.

A successful build publishes the installer, portable ZIP, and `SHA256SUMS` to
Releases. Actions tests the installer by installing it, checking the hashes of
both executables, and uninstalling it. See the
[release guide](docs/elyhorizon-releases.md) for details.

## Build from source

Development requires Rust, Python 3.10+, and JDK 25. Game resource preparation is
described in the [client documentation](docs/native-client.md) and
[CONTRIBUTING.md](CONTRIBUTING.md).

```sh
git clone https://github.com/kkerneldev/Elyhorizon.git
cd Elyhorizon
cargo build --release --locked -p horizon-client --bin horizon-client
cargo build --manifest-path launcher/Cargo.toml --release --locked --bin horizon-launcher
```

Place the built `horizon-launcher.exe` and `horizon-client.exe` in the same folder
to use Ely.by support. Prepared game files are stored separately in the ignored
`.local/` directory; never add them to the repository.

## Issues and contributions

Report problems with Ely.by sign-in, the installer, or changes in this fork in
[Elyhorizon Issues](https://github.com/kkerneldev/Elyhorizon/issues).
Include the build version, the action that triggered the issue, and the error
message. Do not include passwords, tokens, or the OAuth application secret.

Upstream engine documentation: [Horizon](https://github.com/horizon-rust/horizon),
[architecture](docs/architecture.md), [mods](docs/mods.md),
[servers](docs/hosting.md), and [horizon-ctl control](docs/agent-control.md).

## Credits and license

- **Horizon** is the original project by [Ludovic Marie](https://github.com/ludovic111) and [Horizon contributors](https://github.com/horizon-rust/horizon/graphs/contributors).
- **Elyhorizon** is [this fork](https://github.com/kkerneldev/Elyhorizon), adding Ely.by support and its own Windows distribution.
- **Ely.by** is the [account and skin service](https://ely.by) used by the integration.

Source code is distributed under the [MIT license](LICENSE); dependencies retain
their own licenses ([notices](THIRD_PARTY_NOTICES.md)). Horizon and Ely.by logos
belong to their respective projects; see [image sources](docs/media/elyhorizon/README.md).

Elyhorizon is not an official Minecraft product and is not affiliated with Mojang
or Microsoft. Minecraft code and game assets are not distributed in this repository.

<details>
<summary>Upstream Horizon technical audit</summary>

The audit below is preserved from the upstream README. Its date is stated in the
report; the results cover that test suite, not every scenario in this fork.

<p>
  <img src="https://img.shields.io/badge/vanilla%20parity-100%25-F0795A" alt="Upstream vanilla parity 100% of audited tests">
  <img src="https://img.shields.io/badge/player--visible%20gaps-833%20open-F0795A" alt="Upstream player-visible gaps: 833 open">
</p>

## Project status

**Player-visible gaps: 833 open** of the 833 the [port ledger](tools/port_ledger.py) found, reading every vanilla 26.3 method against Horizon's code: vanilla behaviour a player would notice that Horizon doesn't have yet. By area: Screens, HUD and client packets [197](https://github.com/horizon-rust/horizon/issues/3518); Entity and block rendering, models [129](https://github.com/horizon-rust/horizon/issues/3519); Animals [110](https://github.com/horizon-rust/horizon/issues/3512); Server and players on the server [106](https://github.com/horizon-rust/horizon/issues/3520); Blocks, redstone and fluids [86](https://github.com/horizon-rust/horizon/issues/3516); Entities, projectiles, vehicles and decorations [64](https://github.com/horizon-rust/horizon/issues/3514); Monsters and bosses [53](https://github.com/horizon-rust/horizon/issues/3513); Mob AI, villagers and raids [35](https://github.com/horizon-rust/horizon/issues/3521); World, world generation and saves [32](https://github.com/horizon-rust/horizon/issues/3517); Items, enchantments and inventories [21](https://github.com/horizon-rust/horizon/issues/3515). This number goes down as gaps close (`tools/parity_gaps.py`); the harness results below say that what is tested matches.

The table is the project's own audit: every subsystem matches vanilla in every harness that compares it. 100% means 100% of what the harnesses compare, not that nothing else differs.

### Java parity: 100%

Scores: 0 (absent), 0.25 (started), 0.5 (playable, major features missing), 0.75 (mostly complete) or 1 (complete and verified by differential tests); the percentage is their average. Last audited 2026-10-13: all 281 harnesses pass on Linux and macOS. The [port ledger](tools/port_ledger.py) adds a method-by-method view of what is ported.

| Subsystem | Score | Main gaps |
| --- | ---: | --- |
| Window, input and game loop | 1 | None: game loop, options, camera, auto-jump, toggle sprint/sneak and frame limiter match |
| Assets and resources | 1 | None: item models match over 196,290 stacks; the resource pack screen over 3,000 sessions |
| Network protocol and multiplayer | 1 | None: joins vanilla servers; the dedicated server matches vanilla's files, login, configuration, play and command tree; LAN discovery over 462 cases (offline mode only) |
| Chunk storage and palettes | 1 | None: client and saves share paletted containers, identical to vanilla over 266 scenarios |
| Section meshing | 1 | None: 112 scenes and over a million quads identical to vanilla |
| Rendering | 1 | None: entity models, block entity renderers (407 cases) and animations (296), HUD overlays (2,200 states), the recipe book (600), player list, subtitles and world border match |
| Lighting | 1 | None: saved and relit light matches vanilla, slabs and stairs included |
| World generation | 1 | None: 2,352 whole chunks over 225 structure sites, in all three dimensions, identical to vanilla |
| Region files and saves | 1 | None: saves round-trip with vanilla; worlds from 1.8 to 26.2 upgrade as vanilla's DataFixers do |
| Entities and physics | 1 | None: attributes, spawn rules, natural spawning (15,500 cases), raids and patrols (1,900), `finalizeSpawn` (24,000) and mob AI (92 scenarios) match |
| Interaction and inventory | 1 | None: menu clicks over 8,000 sequences; recipe book placement (6,000), buttons (1,200), saved book (1,500), loom and crafter screens (1,600) |
| Menus, HUD and chat | 1 | None: every command (758 over 39,685 inputs), scoreboards (81,000 operations), the player list (872,000 draw calls), chat and its suggestions (3,000 sessions) match |
| Audio | 1 | None: the sound engine, sound events, music, situational music, ambience and subtitles match vanilla |
| Block behaviours and ticking | 1 | None: block updates, random ticks, redstone, dispensers (6,000 cases), crafters (4,000), container spills (4,000), copper golems and the sculk family (435 scenarios) match |

</details>
