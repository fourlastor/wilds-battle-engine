#!/usr/bin/env python3
"""Compare the local PokeAPI source data, PokeWilds JSON and shipped Lua files.

Uses only Python's standard library. PokeAPI builds its database from these CSVs,
so no server, Docker container, public API or database rebuild is needed.
This is an inventory of files, not a validation of move behavior.
"""

import argparse
import csv
import json
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
STATUSES = ("MISSING", "PRESENT", "TO_BE_PORTED")
# Stable PokeAPI move IDs. Generic Z-Moves have separate physical/special rows.
Z_MOVE_IDS = frozenset(range(622, 659)) | frozenset(range(695, 704)) | {719} | frozenset(range(723, 729))
MAX_MOVE_IDS = {743} | frozenset(range(757, 775))  # Includes Max Guard.
SHADOW_MOVE_IDS = frozenset(range(10001, 10019))
LAST_SWORD_SHIELD_ID = 826  # Eerie Spell, The Crown Tundra.
LAST_ARCEUS_ID = 850


def exclusion_reason(move_id):
    if move_id in SHADOW_MOVE_IDS:
        return "SHADOW_MOVE"
    if move_id in Z_MOVE_IDS:
        return "Z_MOVE"
    if move_id in MAX_MOVE_IDS:
        return "MAX_MOVE"
    if LAST_SWORD_SHIELD_ID < move_id <= LAST_ARCEUS_ID:
        return "LEGENDS_ARCEUS"
    if move_id > LAST_SWORD_SHIELD_ID:
        return "AFTER_SWORD_SHIELD"
    return ""


def read_csv(path):
    with path.open(encoding="utf-8", newline="") as stream:
        return list(csv.DictReader(stream))


def insert_unique(target, move_id, entry, source):
    if move_id in target:
        raise ValueError(f"Duplicate move ID {move_id} in {source}")
    target[move_id] = entry


def read_pokeapi(repo):
    data = repo / "data/v2/csv"
    languages = read_csv(data / "languages.csv")
    english = next(row["id"] for row in languages if row["identifier"] == "en")
    names = {}
    for row in read_csv(data / "move_names.csv"):
        if row["local_language_id"] == english:
            insert_unique(names, int(row["move_id"]), row["name"], "PokeAPI English names")
    moves = {}
    for row in read_csv(data / "moves.csv"):
        move_id = int(row["id"])
        insert_unique(moves, move_id, {
            "slug": row["identifier"],
            "name": names.get(move_id, row["identifier"]),
        }, "PokeAPI moves.csv")
    if not moves:
        raise ValueError("PokeAPI moves.csv is empty")
    return moves


def read_pokewilds(directory):
    moves = {}
    # index.json is a registry, not a move definition.
    for path in sorted(directory.glob("*.json")):
        if not path.stem.isdecimal():
            continue
        entry = json.loads(path.read_text(encoding="utf-8"))
        move_id = int(entry["ID"])
        if move_id != int(path.stem):
            raise ValueError(f"PokeWilds ID {move_id} does not match filename {path}")
        insert_unique(moves, move_id, {
            "slug": entry["Name"], "name": entry["HumanName"], "file": path.name,
        }, directory)
    if not moves:
        raise ValueError(f"No numeric move JSON files in {directory}")
    return moves


def slug_key(slug):
    return slug.lower().replace("_", "-")


def read_engine(root, pokeapi, pokewilds):
    manifest = json.loads((root / "moves/catalog.json").read_text(encoding="utf-8"))
    metadata = {}
    declared_files = {}
    aliases = defaultdict(set)
    for source in (pokeapi, pokewilds):
        for move_id, entry in source.items():
            aliases[slug_key(entry["slug"])].add(move_id)
    for entry in manifest["moves"]:
        move_id = int(entry["id"])
        insert_unique(metadata, move_id, entry, "engine manifest")
        path = (root / entry["file"]).resolve()
        if path in declared_files:
            raise ValueError(f"Multiple manifest entries refer to {path}")
        declared_files[path] = move_id
        aliases[slug_key(entry["slug"])].add(move_id)

    directory = root / "moves"
    files = sorted(directory.glob("*.lua"))
    if not files:
        raise ValueError(f"No Lua move files in {directory}")
    system = root / "crates/wilds-battle-engine/src/system_moves.lua"
    if system.is_file():
        files.append(system)
    present = {}
    for path in files:
        # Numeric manifest IDs handle renamed slugs; only existing files count.
        move_id = declared_files.get(path.resolve())
        if move_id is None:
            key = "struggle" if path == system else slug_key(path.stem)
            candidates = aliases.get(key, set())
            if len(candidates) != 1:
                raise ValueError(f"Cannot map {path} to one move ID: {sorted(candidates)}")
            move_id = next(iter(candidates))
        insert_unique(present, move_id, str(path.relative_to(root)), "engine Lua files")
    return present, metadata


def audit(pokeapi_repo, pokewilds_directory, engine_root):
    pokeapi = read_pokeapi(pokeapi_repo)
    pokewilds = read_pokewilds(pokewilds_directory)
    engine, metadata = read_engine(engine_root, pokeapi, pokewilds)
    rows = []
    # Keep locally present IDs even if this PokeAPI snapshot lacks their data.
    for move_id in sorted(pokeapi.keys() | pokewilds.keys() | engine.keys()):
        entry = pokewilds.get(move_id) or pokeapi.get(move_id) or metadata[move_id]
        reason = exclusion_reason(move_id)
        if reason:
            status = "EXCLUDED"
        elif move_id in engine:
            status = "PRESENT"
        elif move_id in pokewilds:
            status = "TO_BE_PORTED"
        else:
            status = "MISSING"
        rows.append({
            "id": move_id,
            "slug": entry["slug"],
            "name": entry["name"],
            "status": status,
            "exclusion_reason": reason,
            "in_pokeapi": move_id in pokeapi,
            "pokewilds_file": pokewilds.get(move_id, {}).get("file", ""),
            "engine_file": engine.get(move_id, ""),
        })
    counts = Counter(row["status"] for row in rows)
    exclusions = Counter(row["exclusion_reason"] for row in rows if row["exclusion_reason"])
    return {
        "sources": {
            "pokeapi_moves": len(pokeapi),
            "pokewilds_moves": len(pokewilds),
            "engine_moves": len(engine),
        },
        "counts": {status: counts[status] for status in STATUSES},
        "exclusions": dict(sorted(exclusions.items())),
        "rows": rows,
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pokeapi-dir", type=Path, default=Path.home() / "Documents/pokeapi")
    parser.add_argument("--pokewilds-moves", type=Path,
                        default=ROOT.parent / "pokewilds-next/pokemon_data/assets/moves")
    parser.add_argument("--engine-root", type=Path, default=ROOT)
    parser.add_argument("--output", type=Path, help="Write every included/excluded move to a CSV")
    args = parser.parse_args(argv)
    try:
        report = audit(args.pokeapi_dir.expanduser(), args.pokewilds_moves.expanduser(),
                       args.engine_root.expanduser())
        if args.output:
            output = args.output.expanduser()
            output.parent.mkdir(parents=True, exist_ok=True)
            with output.open("w", encoding="utf-8", newline="") as stream:
                writer = csv.DictWriter(stream, fieldnames=list(report["rows"][0]),
                                        lineterminator="\n")
                writer.writeheader()
                writer.writerows(report["rows"])
    except (OSError, ValueError, KeyError, StopIteration) as error:
        parser.exit(1, f"Move audit failed: {error}\n")
    print("Source inventory:")
    for source, count in report["sources"].items():
        print(f"  {source}: {count}")
    print("\nSword/Shield scope (file presence, not behavior validation):")
    for status, count in report["counts"].items():
        print(f"  {status}: {count}")
    print(f"  TOTAL: {sum(report['counts'].values())}")
    print(f"\nEXCLUDED: {sum(report['exclusions'].values())}")
    for reason, count in report["exclusions"].items():
        print(f"  {reason}: {count}")
    for status in ("MISSING", "TO_BE_PORTED"):
        for row in report["rows"]:
            if row["status"] == status:
                print(f"{status}\t{row['id']}\t{row['slug']}\t{row['name']}")
    outside_scope = [row for row in report["rows"]
                     if row["status"] == "EXCLUDED" and row["engine_file"]]
    if outside_scope:
        print("\nWARNING: excluded moves still have engine files:")
        for row in outside_scope:
            print(f"  {row['id']} {row['engine_file']}")
    if args.output:
        print(f"\nCSV: {args.output.expanduser().resolve()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
