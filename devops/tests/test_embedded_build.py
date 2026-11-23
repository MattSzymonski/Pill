#!/usr/bin/env python3
"""
Builds the embedded image, or skips when this machine cannot.

PROTECTS: The shipping binary still compiles, links and fits its size budget
as a standalone ESP32-C6 image, the image fits the flash layout the build
declares for the board, the C# refusal still holds.
TIER: build

REQUIREMENTS
  - Python 3.8+
  - The espup `esp` Rust toolchain, ESP-IDF 5.1.3 with the riscv32-esp-elf
    toolchain, `ldproxy` and `espflash` on PATH, and `ESP_IDF_PATH` set.
    Without them this suite prints [SKIP] and exits 0.

DESCRIPTION
    Runs `devops/ci_cd/build_shipping.py` for `riscv32imac-esp-espidf` on the
    headless example. That covers the embedded posture end to end as far as a
    machine without the board can: the dependency graph resolves, std is built
    from source, the image links through `ldproxy` against the ESP-IDF runtime,
    and the flash and RAM budgets in the script are enforced.

    Then it reads what the build itself declares and checks that a bare board
    could be flashed from it: the factory app partition in the generated
    partition table must hold the image, the declared flash size must cover the
    board's flash chip, and the shipped bundle must carry that partition table.
    A budget alone cannot catch a layout that does not fit a fresh board.

    Nothing here needs the board: flashing and the boot log are the hardware
    check, described in the item that added the embedded posture.

USAGE
  python devops/tests/test_embedded_build.py

EXAMPLE USAGE
  python devops/tests/test_embedded_build.py
"""

# Standard library
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import List, Optional

# =============================================================================
# Constants
# =============================================================================

REPOSITORY_ROOT = Path(__file__).resolve().parent.parent.parent
TARGET = "riscv32imac-esp-espidf"
EXAMPLE_PROJECT = "examples/headless/headless_rust"
IDF_VERSION = "v5.1.3"

# The board's flash chip. The build has to declare at least this much, or the
# image is flashed into a chip the tooling believes is smaller.
EMBEDDED_FLASH_BYTES = 8 * 1024 * 1024

# The app type and factory subtype are ESP-IDF's constants for the partition the
# app image is flashed into.
PARTITION_TYPE_APP = 0x00
PARTITION_SUBTYPE_FACTORY = 0x00


# =============================================================================
# Free Functions
# =============================================================================

def has_esp_toolchain() -> bool:
    """Whether rustup has the espup `esp` toolchain installed."""
    try:
        completed = subprocess.run(
            ["rustup", "toolchain", "list"], capture_output=True, text=True
        )
    except OSError:
        return False
    return any(
        line.split()[0] == "esp"
        for line in completed.stdout.splitlines()
        if line.strip()
    )


def skip_reason() -> str:
    """Why this machine cannot build the image, or an empty string."""
    if not has_esp_toolchain():
        return "the espup `esp` toolchain is not installed (rustup toolchain list)"
    idf_path = os.environ.get("ESP_IDF_PATH", "")
    if not idf_path:
        return f"ESP_IDF_PATH is not set (point it at an ESP-IDF {IDF_VERSION} checkout)"
    if not Path(idf_path).is_dir():
        return f"ESP_IDF_PATH points at {idf_path}, which is not a directory"
    return ""


def embedded_build_directory() -> Path:
    """The target directory an embedded build uses, mirroring build_shipping.py."""
    override = os.environ.get("PILL_EMBEDDED_TARGET_DIR")
    if override:
        return Path(override)
    return Path(f"{REPOSITORY_ROOT.drive or 'C:'}\\p")


def newest(paths) -> Optional[Path]:
    """The most recently modified of these paths, or None when there are none."""
    candidates = [path for path in paths if path.exists()]
    if not candidates:
        return None
    return max(candidates, key=lambda path: path.stat().st_mtime)


def shipping_build():
    """The build script, for the parsers the build and this suite share."""
    sys.path.insert(0, str(REPOSITORY_ROOT / "devops" / "ci_cd"))
    import build_shipping

    return build_shipping


def read_partition_table(path: Path) -> List[dict]:
    """The entries of an ESP-IDF partition table binary.

    Read by the build's own parser: a second one here could disagree with the
    build about the partition the image has to fit, which is the whole check.
    """
    return shipping_build().read_partition_table(path)


def declared_flash_bytes(sdkconfig: Path) -> Optional[int]:
    """The flash size the build declares, from its generated sdkconfig."""
    text = sdkconfig.read_text(encoding="utf-8", errors="replace")
    match = re.search(r'CONFIG_ESPTOOLPY_FLASHSIZE="(\d+)MB"', text)
    return int(match.group(1)) * 1024 * 1024 if match else None


def image_flash_bytes(elf_path: Path) -> Optional[int]:
    """The flash the linked image occupies, measured as the size gate measures it."""
    build_shipping = shipping_build()
    size_tool = build_shipping.find_embedded_size_tool(TARGET)
    if size_tool is None:
        return None
    sections = build_shipping.embedded_size_sections(elf_path, size_tool)
    return sections["flash_bytes"] if sections else None


def shipped_bundle() -> Optional[Path]:
    """The dated bundle the build wrote for the example project."""
    bundles = (REPOSITORY_ROOT / EXAMPLE_PROJECT / "build").glob("*/")
    return newest(path for path in bundles if path.is_dir())


def layout_problems(build_directory: Path) -> List[str]:
    """Everything that would stop a bare board from being flashed and booted."""
    partition_table = newest(build_directory.rglob("partition-table.bin"))
    if partition_table is None:
        return [f"the build produced no partition table under {build_directory}"]
    factory = [
        entry
        for entry in read_partition_table(partition_table)
        if entry["type"] == PARTITION_TYPE_APP
        and entry["subtype"] == PARTITION_SUBTYPE_FACTORY
    ]
    if not factory:
        return [f"{partition_table} declares no factory app partition"]
    room = factory[0]["size"]

    problems = []
    bundle = shipped_bundle()
    # The image that ships is the one in the bundle; the target directory also
    # holds ESP-IDF's own object dumps (`libespidf.elf`), which are not it.
    elf_path = newest(bundle.glob("*.elf")) if bundle is not None else None
    flash_bytes = image_flash_bytes(elf_path) if elf_path else None
    if flash_bytes is None:
        problems.append("the shipped image's size could not be measured")
    elif flash_bytes > room:
        problems.append(
            f"the image needs {flash_bytes / 1024:.0f} KB of flash but the "
            f"factory partition the build declares is {room / 1024:.0f} KB"
        )

    sdkconfig = newest(build_directory.rglob("sdkconfig"))
    flash_declaration = declared_flash_bytes(sdkconfig) if sdkconfig else None
    if flash_declaration is None:
        problems.append("the build declares no flash size against the chip")
    elif flash_declaration < EMBEDDED_FLASH_BYTES:
        problems.append(
            f"the build declares {flash_declaration // (1024 * 1024)} MB of "
            f"flash but the board's chip is {EMBEDDED_FLASH_BYTES // (1024 * 1024)} MB"
        )

    if bundle is None:
        problems.append("the build wrote no bundle for the example project")
    elif not any(bundle.glob("*partition*.bin")):
        problems.append(
            f"the bundle {bundle.name} ships no partition table, so a bare "
            "board has nothing to flash the layout from"
        )
    return problems


def main() -> int:
    """Build the embedded image, or skip with a message. Returns an exit code."""
    reason = skip_reason()
    if reason:
        print(f"[SKIP] embedded build: {reason}")
        return 0
    print(f"[RUN] embedded build for {TARGET} from {EXAMPLE_PROJECT}")
    completed = subprocess.run(
        [
            sys.executable,
            str(REPOSITORY_ROOT / "devops" / "ci_cd" / "build_shipping.py"),
            "--project",
            EXAMPLE_PROJECT,
            "--target",
            TARGET,
        ],
        cwd=str(REPOSITORY_ROOT),
    )
    # The build regenerates the tracked bundle for this project; put the stub
    # back so the tree is left as it was found, whatever the build's outcome.
    subprocess.run(
        [
            sys.executable,
            str(REPOSITORY_ROOT / "devops" / "tools" / "generate_shipping_bundle.py"),
            "--restore-stub",
        ],
        cwd=str(REPOSITORY_ROOT),
    )
    if completed.returncode != 0:
        print("[FAIL] the embedded image did not build")
        return completed.returncode
    print("[RUN] embedded flash layout")
    problems = layout_problems(embedded_build_directory())
    for problem in problems:
        print(f"  - {problem}")
    if problems:
        print("[FAIL] the embedded image does not fit the flash layout it declares")
        return 1
    print("[PASS] the embedded image built and fits its budget and its flash layout")
    return 0


if __name__ == "__main__":
    sys.exit(main())
