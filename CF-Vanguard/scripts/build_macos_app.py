#!/usr/bin/env python3
"""Build a self-contained macOS app with the real multi-resolution game icon."""
from __future__ import annotations

import argparse
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--debug", action="store_true", help="Package the faster debug build instead of release")
    args = parser.parse_args()
    if sys.platform != "darwin":
        parser.error("The macOS bundle must be built on macOS.")
    command = ["cargo", "build"]
    if not args.debug:
        command.append("--release")
    subprocess.run(command, cwd=ROOT, check=True)
    profile = "debug" if args.debug else "release"
    bundle = ROOT / "dist/Cardfight Vanguard.app"
    contents = bundle / "Contents"
    executable = contents / "MacOS"
    resources = contents / "Resources"
    executable.mkdir(parents=True, exist_ok=True)
    resources.mkdir(parents=True, exist_ok=True)
    shutil.copy2(ROOT / f"target/{profile}/vanguard_story", executable / "vanguard_story")
    shutil.copy2(ROOT / "assets/branding/AppIcon.icns", resources / "AppIcon.icns")
    # Character/branding/tabletop art is embedded; the card image cache ships beside it.
    shutil.copytree(ROOT / "assets/card_images", resources / "assets/card_images", dirs_exist_ok=True)
    metadata = {
        "CFBundleExecutable": "vanguard_story",
        "CFBundleIdentifier": "local.cfvanguard.limitbreak",
        "CFBundleName": "Cardfight Vanguard",
        "CFBundleDisplayName": "Cardfight!! Vanguard",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": "0.1.0",
        "CFBundleVersion": "0.1.0",
        "CFBundleIconFile": "AppIcon.icns",
        "CFBundleIconName": "AppIcon",
        "NSHighResolutionCapable": True,
        "LSApplicationCategoryType": "public.app-category.card-games",
    }
    with (contents / "Info.plist").open("wb") as handle:
        plistlib.dump(metadata, handle)
    (contents / "PkgInfo").write_bytes(b"APPL????")
    print(f"Built {bundle}")


if __name__ == "__main__":
    main()
