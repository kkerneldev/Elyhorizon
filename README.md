<p align="center">
  <a href="https://github.com/horizon-rust/horizon"><img src="docs/media/brand/icon.svg" alt="Логотип Horizon" width="104" height="104"></a>
  &nbsp;&nbsp;&nbsp;&nbsp;
  <a href="https://ely.by"><img src="docs/media/elyhorizon/elyby-logo.png" alt="Логотип Ely.by" width="104" height="104"></a>
</p>

<h1 align="center">Elyhorizon</h1>

<p align="center">
  <b>Horizon с аккаунтами Ely.by и своим установщиком для Windows.</b><br>
  Независимый форк <a href="https://github.com/horizon-rust/horizon">Horizon</a> — клиента Minecraft: Java Edition 26.3, написанного на Rust.
</p>

<p align="center">
  <a href="https://github.com/kkerneldev/Elyhorizon/actions/workflows/elyhorizon-release.yml"><img src="https://github.com/kkerneldev/Elyhorizon/actions/workflows/elyhorizon-release.yml/badge.svg" alt="Сборка Elyhorizon"></a>
  <img src="https://img.shields.io/badge/Windows-x64-5b3fd6" alt="Windows x64">
  <img src="https://img.shields.io/badge/Ely.by-Browser%20OAuth-208260" alt="Вход Ely.by через браузер">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-5b3fd6" alt="Лицензия MIT"></a>
</p>

<p align="center">
  <a href="https://github.com/kkerneldev/Elyhorizon/releases"><b>Скачать установщик</b></a> ·
  <a href="docs/elyby-account.md"><b>Настройка Ely.by</b></a> ·
  <a href="docs/elyhorizon-releases.md"><b>Сборка релиза</b></a> ·
  <a href="https://github.com/kkerneldev/Elyhorizon/issues"><b>Сообщить об ошибке</b></a> ·
  <a href="https://github.com/horizon-rust/horizon"><b>Оригинальный Horizon</b></a>
</p>

## Что это за форк

**Elyhorizon — отдельная сборка Horizon с интеграцией Ely.by.** В основе остаётся
исходный Rust-клиент Horizon. Этот форк добавляет вход в аккаунт Ely.by через
браузер, передачу ника и UUID в игру, загрузку скинов и Windows-установщик, который
ставит лаунчер вместе с подходящей версией клиента.

Проект развивается независимо от команд Horizon и Ely.by. Логотипы обозначают
проекты, на которых основана интеграция; они не означают официальное партнёрство.

| Возможность | Что есть в Elyhorizon |
| --- | --- |
| Аккаунты Ely.by | Вход через системный браузер, выбор аккаунта и обновление сессии |
| Ник и скин | Данные выбранного аккаунта передаются в Horizon; скины загружаются из Ely.by |
| Minecraft Java | Для Java-запусков с Ely.by подключается authlib-injector |
| Установка | Один установщик для лаунчера и Rust-клиента Horizon |
| Портативный запуск | ZIP с обоими EXE для запуска без установки |
| Сборка релизов | GitHub Actions собирает Windows x64, проверяет установку и публикует файлы |

## Скачать и установить

1. Открой [Releases этого форка](https://github.com/kkerneldev/Elyhorizon/releases).
2. Скачай **`Elyhorizon-<версия>-windows-x64-setup.exe`**.
3. Запусти установщик, выбери папку и при желании добавь ярлык на рабочий стол.
4. Открой **Elyhorizon** из меню «Пуск», добавь аккаунт Ely.by и выбери его перед запуском.

Установщик работает для текущего пользователя без прав администратора. Он ставит
оба EXE рядом и добавляет удаление через список установленных приложений Windows.
Понадобится Windows x64 и видеокарта, подходящая для Horizon.

Для портативного запуска скачай `Elyhorizon-<версия>-windows-x64.zip`, распакуй
**всю папку** и запусти `horizon-launcher.exe`. Файлы `horizon-launcher.exe` и
`horizon-client.exe` должны оставаться рядом: так лаунчер сможет использовать
клиент с поддержкой Ely.by. Импортированные сборки и сборки с нативными модами
используют собственный исполняемый файл.

Если файлов для скачивания ещё нет, сначала запусти
[релизный workflow](docs/elyhorizon-releases.md) в своём GitHub-репозитории.

## Вход через Ely.by

Авторизация открывается на сайте Ely.by в твоём браузере. Пароль и код двухфакторной
аутентификации вводятся на сайте; токены сохраняются в системном хранилище учётных
данных.

Для браузерного входа нужно один раз настроить OAuth-приложение на компьютере,
где используется лаунчер. Секрет приложения не входит в релизный EXE и не
переносится с другого компьютера автоматически.
[Инструкция по настройке и командам аккаунта](docs/elyby-account.md).

Ely.by отвечает за аккаунты и текстуры. Файлы Minecraft, библиотеки и ресурсы
загружаются из официальных источников либо берутся из существующей установки.
Они не входят в установщик Elyhorizon.

Аккаунт Ely.by подходит для серверов, настроенных на авторизацию Ely.by.
Доступ к серверам с авторизацией Microsoft зависит от требований самого сервера.

## Как выглядит Horizon

Скриншоты исходного Horizon показывают базовый интерфейс и игру, на которых
построен этот форк.

<p align="center">
  <img src="docs/media/launcher-play.webp" alt="Главная страница лаунчера Horizon" width="780">
</p>

<p align="center">
  <img src="docs/media/sunset.webp" alt="Мир Minecraft в Rust-клиенте Horizon: закат над деревней" width="780">
</p>

## Своя сборка в GitHub Actions

Открой **Actions → Elyhorizon Windows release → Run workflow**, укажи новую
версию, например `0.2.1`, и включи `publish`. Также можно отправить тег
`elyhorizon-v0.2.1`.

После успешной сборки в Releases появятся установщик, портативный ZIP и
`SHA256SUMS`. Сам установщик проверяется в Actions: установка, совпадение хешей
обоих EXE и удаление. Подробности — в
[инструкции по релизам](docs/elyhorizon-releases.md).

## Build from source

Для разработки нужны Rust, Python 3.10+ и JDK 25. Подготовка игровых ресурсов
описана в [документации клиента](docs/native-client.md) и
[CONTRIBUTING.md](CONTRIBUTING.md).

```sh
git clone https://github.com/kkerneldev/Elyhorizon.git
cd Elyhorizon
cargo build --release --locked -p horizon-client --bin horizon-client
cargo build --manifest-path launcher/Cargo.toml --release --locked --bin horizon-launcher
```

Для установки с поддержкой Ely.by положи собранные `horizon-launcher.exe` и
`horizon-client.exe` в одну папку. Подготовленные игровые файлы хранятся отдельно
в игнорируемой `.local/`; их нельзя добавлять в репозиторий.

## Ошибки и участие

Проблемы со входом Ely.by, установщиком и изменениями этого форка можно описать
в [Issues Elyhorizon](https://github.com/kkerneldev/Elyhorizon/issues).
Укажи версию сборки, действие, после которого возникает ошибка, и текст сообщения.
Пароли, токены и секрет OAuth-приложения в отчёт не добавляй.

Документация исходного движка: [Horizon](https://github.com/horizon-rust/horizon),
[архитектура](docs/architecture.md), [моды](docs/mods.md),
[серверы](docs/hosting.md), [управление через horizon-ctl](docs/agent-control.md).

## Авторы и лицензия

- **Horizon** — исходный проект [Ludovic Marie](https://github.com/ludovic111) и [участников Horizon](https://github.com/horizon-rust/horizon/graphs/contributors).
- **Elyhorizon** — [этот форк](https://github.com/kkerneldev/Elyhorizon), поддержка Ely.by и собственная Windows-сборка.
- **Ely.by** — [сервис аккаунтов и скинов](https://ely.by), используемый интеграцией.

Исходный код распространяется по [лицензии MIT](LICENSE); зависимости сохраняют
свои лицензии ([уведомления](THIRD_PARTY_NOTICES.md)). Логотипы Horizon и Ely.by
принадлежат соответствующим проектам; [источники изображений](docs/media/elyhorizon/README.md).

Elyhorizon не является официальным продуктом Minecraft и не связан с Mojang или
Microsoft. Код и игровые ресурсы Minecraft не распространяются в этом репозитории.

<details>
<summary>Технический аудит исходного Horizon</summary>

Ниже сохранён аудит upstream из исходного README. Дата проверки указана в самом
отчёте; результаты относятся к его набору тестов, а не к каждому сценарию этого форка.

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
