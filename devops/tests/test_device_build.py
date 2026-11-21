#!/usr/bin/env python3
"""
Builds the device image, or skips when this machine cannot.

PROTECTS: The shipping binary still compiles, links and fits its size budget
as a standalone ESP32-C6 image, and the C# refusal still holds.
TIER: build

REQUIREMENTS
  - Python 3.8+
  - The espup `esp` Rust toolchain, ESP-IDF 5.1.3 with the riscv32-esp-elf
    toolchain, `ldproxy` and `espflash` on PATH, and `ESP_IDF_PATH` set.
    Without them this suite prints [SKIP] and exits 0.

DESCRIPTION
    Runs `devops/ci_cd/build_shipping.py` for `riscv32imac-esp-espidf` on the
    headless example. That covers the device posture end to end as far as a
    machine without the board can: the dependency graph resolves, std is built
    from source, the image links through `ldproxy` against the ESP-IDF runtime,
    and the flash and RAM budgets in the script are enforced.

    Nothing here needs the board: flashing and the boot log are the hardware
    check, described in the item that added the device posture.

USAGE
  python devops/tests/test_device_build.py

EXAMPLE USAGE
  python devops/tests/test_device_build.py
"""

# Standard library
import os
import subprocess
import sys
from pathlib import Path

# =============================================================================
# Constants
# =============================================================================

REPOSITORY_ROOT = Path(__file__).resolve().parent.parent.parent
TARGET = "riscv32imac-esp-espidf"
EXAMPLE_PROJECT = "examples/headless/headless_rust"
IDF_VERSION = "v5.1.3"


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


def main() -> int:
    """Build the device image, or skip with a message. Returns an exit code."""
    reason = skip_reason()
    if reason:
        print(f"[SKIP] device build: {reason}")
        return 0
    print(f"[RUN] device build for {TARGET} from {EXAMPLE_PROJECT}")
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
        print("[FAIL] the device image did not build")
        return completed.returncode
    print("[PASS] the device image built and fits its size budget")
    return 0


if __name__ == "__main__":
    sys.exit(main())
