<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://pillengine.org/logos/pill_logo_white.svg">
    <img src="https://pillengine.org/logos/pill_logo_black.svg" width="350">
  </picture>
</p>

<br/>

Modern, free and blazingly fast game engine. Built for developers who refuse to compromise.  

Designed from the ground up around performance, modularity and a frictionless developer experience, it gives creators the freedom to build anything from tiny experimental projects to massive, complex worlds - without unnecessary bloat getting in the way.

> **NOTE: Pill is currently under heavy architecture rework in order to deliver all flagship features.**  
> **💊 v1.0 ETA - Q1/2027**  

## Flagship Feature Goals
- **100% free and open source**
- **C# and Rust scripting languages**
- **Hot reloading pushed to the max**
- **Crash-resistant with full project sandboxing**
- **Hybrid Entity Component System architecture**
- **Proven performance**
- **Anti-bloat, modular architecture**
- **Next-level error logs**
- **Human-readable asset format**
- **Live game + scene editing workflow**
- **Tiny build sizes**
- **Absolute streaming beast**
- **Blazing startup times**
- **Embedded support (Rust projects, headless, ESP32-C6 today)**

## Embedded builds

A Rust project can ship as one standalone headless image for the ESP32-C6
(`riscv32imac-esp-espidf`): no renderer, no development host, no C# runtime.
The ESP-IDF runtime links with it and `app_main` runs the same project the
desktop build runs.

The frontend for it is its own crate, `modules/frontends/pill_embedded`, beside
the desktop's. It has no development posture at all - a board has no hot reload,
so "shipping only" is its dependency graph rather than a check - and it picks
its SDK with a binary: `src/bin/espidf_target.rs`, built only when the crate's `espidf`
feature is on. That binary supplies the four pieces the SDK owns - the entry
point, the worker pool, the frame pacing and the heap probe. The shared start-up
and the project setup come from `pill_player`; the frame loop and the heap
report from this crate. The crate is its own workspace and is excluded from
`modules/`, because its only code links `esp-idf-sys`, which no host build
resolves.

```shell
set ESP_IDF_PATH=C:\Users\<you>\.espressif\esp-idf\v5.1.3
python devops/ci_cd/build_shipping.py --project examples/headless/headless_rust --target riscv32imac-esp-espidf
```

The build writes a dated bundle with everything a bare board needs: the image,
the bootloader and the partition table ESP-IDF generated for it, the layout
behind that table, and the license notices. Flash the whole bundle - with the
image alone the flashing tool supplies a bootloader and a partition table of its
own, and the installed layout is then not the built one:

```shell
set BUNDLE=examples\headless\headless_rust\build\<date>
espflash erase-flash --chip esp32c6 --port COM4 --before default-reset
espflash flash --chip esp32c6 --port COM4 --flash-size 8mb ^
  --bootloader %BUNDLE%\bootloader.bin --partition-table %BUNDLE%\memory_partitions_config.csv ^
  --monitor %BUNDLE%\HeadlessRust.elf
```

The layout is `modules/frontends/pill_embedded/res/esp32/memory_partitions_config.csv`: `nvs`
24 KB, `phy_init` 4 KB, a 2 MB factory app and a 1 MB `data` partition for
project content, on the C6's 8 MB of flash. The flash budget is 90 % of that
factory partition, computed from the table the bundle ships, and the RAM budget
is an absolute 97.7 KB. The headless example measures a 1.4 MB flash image on
the board (a 1.5 MB app, 72 % of its partition) and 46.7 KB of internal RAM.

What the build needs: the espup `esp` Rust toolchain, ESP-IDF 5.1.3 with the
`riscv32-esp-elf` toolchain, `ldproxy` and `espflash` on `PATH`, the ESP-IDF
environment exported (`. <esp-idf>\export.ps1` here, `export.sh` elsewhere) and
`ESP_IDF_PATH` pointing at the same checkout - or, without the export,
`IDF_PATH` at the checkout and `IDF_TOOLS_PATH` at the directory holding its
`tools/` and `python_env/`. A C# project is refused for an embedded target
before anything is built.

`devops/tests/test_embedded_build.py` builds the image, runs the frontend's
clippy - the only place its warnings are seen, because the host workspace
cannot compile it - and checks the image against the flash layout it declares.
Without that environment it prints `[SKIP]` and exits 0, so read its log rather
than its exit code when you expected a build.

The embedded image is Rust-only. There is no CLR and no NativeAOT for these
chips, so scripting on them is not offered.

## Links
**Discover Pill at [PillEngine.org](https://pillengine.org)**

**Join the community on [Discord Server](https://discord.gg/VUKNQrctms)**

<!-- **Quick start with [Pill Guide](https://docs.pillengine.org/guide)** -->

**Check demos at [Pill Demos](https://pillengine.org/demos)**

---

<br/>
<br/>
<br/>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://pillengine.org/logos/pill_pile_white.png">
    <img src="https://pillengine.org/logos/pill_pile_black.png" width="650">
  </picture>
</p>
