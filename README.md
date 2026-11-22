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
- **Embedded device support (Rust projects, headless, ESP32-C6 today)**

## Device builds

A Rust project can ship as one standalone headless image for the ESP32-C6
(`riscv32imac-esp-espidf`): no renderer, no development host, no C# runtime.
The ESP-IDF runtime links with it and `app_main` runs the same project the
desktop build runs.

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
  --bootloader %BUNDLE%\bootloader.bin --partition-table %BUNDLE%\partitions.csv ^
  --monitor %BUNDLE%\HeadlessRust.elf
```

The layout is `modules/frontends/pill_standalone/device/partitions.csv`: `nvs`
24 KB, `phy_init` 4 KB, a 2 MB factory app and a 1 MB `data` partition for
project content, on the C6's 8 MB of flash. The flash budget is 90 % of that
factory partition, computed from the table the bundle ships, and the RAM budget
is an absolute 97.7 KB. The headless example measures a 1.4 MB flash image on
the device (a 1.5 MB app, 72 % of its partition) and 46.7 KB of internal RAM.

What the build needs: the espup `esp` Rust toolchain, ESP-IDF 5.1.3 with the
`riscv32-esp-elf` toolchain, and `ldproxy` and `espflash` on `PATH`. A C#
project is refused for a device target before anything is built.

The device image is Rust-only. There is no CLR and no NativeAOT for these
chips, so scripting on a device is not offered.

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
