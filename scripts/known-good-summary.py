#!/usr/bin/env python3
"""Print the latest Known-good set of known-good.toml, one Component per line (for release notes)."""
import sys
import tomllib

path = sys.argv[1] if len(sys.argv) > 1 else "known-good.toml"
with open(path, "rb") as f:
    sets = [s for s in tomllib.load(f)["set"] if s["status"] == "known-good"]
latest = sets[-1]
print("known-good set", latest["id"])
for name, entry in latest["components"].items():
    print(f"  {name:<13} {entry['version']}")
