#!/usr/bin/python3
import os
import subprocess
import sys

from pathlib import Path

if len(sys.argv) < 2:
    print(f"Usage: {sys.argv[0]} <path>", file=sys.stderr)
    exit(1)

rumorph_home_path = Path(sys.argv[1])

# Sanity check
if rumorph_home_path.exists():
    print(f"Error: {rumorph_home_path} already exists", file=sys.stderr)
    exit(1)

# match directory names with the rumorph runner
rumorph_home_path.mkdir()

cargo_home_path = rumorph_home_path / "cargo_home"
cargo_home_path.mkdir()

sccache_home_path = rumorph_home_path / "sccache_home"
sccache_home_path.mkdir()

rumorph_cache_path = rumorph_home_path / "rumorph_cache"
rumorph_cache_path.mkdir()

campaign_path = rumorph_home_path / "campaign"
campaign_path.mkdir()