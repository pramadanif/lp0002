#!/usr/bin/env python3
"""Replace the root manifest.json inside an .lgx archive.

An .lgx is a gzipped tar. `lgx add` writes its own manifest from the packaging metadata, which
drops the fields we filled in (author, licence, homepage, the macOS targets). This puts ours back.

Referenced by scripts/build-basecamp.sh. Modelled on the helper SPEL's own scaffold generates.

    patch_lgx_manifest.py <pkg.lgx> <manifest.json>
"""
import gzip
import io
import json
import shutil
import sys
import tarfile


def main() -> int:
    if len(sys.argv) != 3:
        print(f"usage: {sys.argv[0]} <pkg.lgx> <manifest.json>", file=sys.stderr)
        return 2
    lgx_path, manifest_path = sys.argv[1], sys.argv[2]

    with open(manifest_path, "rb") as f:
        manifest = f.read()
    try:
        json.loads(manifest)
    except json.JSONDecodeError as e:
        print(f"FATAL: {manifest_path} is not valid JSON: {e}", file=sys.stderr)
        return 1

    with gzip.open(lgx_path, "rb") as gz:
        raw = gz.read()

    out = io.BytesIO()
    replaced = False
    with tarfile.open(fileobj=io.BytesIO(raw)) as src, \
         tarfile.open(fileobj=out, mode="w") as dst:
        for member in src.getmembers():
            if member.name.lstrip("./") == "manifest.json":
                member.size = len(manifest)
                dst.addfile(member, io.BytesIO(manifest))
                replaced = True
            else:
                extracted = src.extractfile(member)
                dst.addfile(member, extracted)

    if not replaced:
        print("FATAL: no manifest.json at the root of the archive", file=sys.stderr)
        return 1

    shutil.copyfile(lgx_path, lgx_path + ".bak")
    with gzip.open(lgx_path, "wb") as gz:
        gz.write(out.getvalue())
    print(f"Patched {lgx_path}: manifest.json replaced from {manifest_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
