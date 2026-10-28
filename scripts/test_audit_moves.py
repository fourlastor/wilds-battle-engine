"""Isolated source fixtures exercise audit joins, exclusions and failure cases."""

import csv
import json
import tempfile
import unittest
from pathlib import Path

from audit_moves import audit, exclusion_reason


class MoveAuditTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.api = self.root / "pokeapi"
        self.pw = self.root / "pokewilds"
        self.engine = self.root / "engine"
        self.data = self.api / "data/v2/csv"
        self.data.mkdir(parents=True)
        self.pw.mkdir()
        (self.engine / "moves").mkdir(parents=True)
        self.csv("languages", ["id", "identifier"], [[9, "en"]])
        self.csv("move_names", ["move_id", "local_language_id", "name"], [])
        self.manifest([])
        self.local_move(1, "pound")
        # Must not be mistaken for an individual move JSON.
        (self.pw / "index.json").write_text('{"Entries": []}')

    def csv(self, name, header, rows):
        with (self.data / f"{name}.csv").open("w", newline="") as stream:
            writer = csv.writer(stream)
            writer.writerow(header)
            writer.writerows(rows)

    def api_moves(self, rows):
        self.csv("moves", ["id", "identifier"], rows)

    def local_move(self, move_id, slug):
        (self.pw / f"{move_id}.json").write_text(json.dumps({
            "ID": move_id, "Name": slug, "HumanName": slug.title(),
        }))

    def lua(self, slug):
        (self.engine / "moves" / f"{slug}.lua").write_text(f'return {{id="{slug}"}}')

    def manifest(self, entries):
        (self.engine / "moves/catalog.json").write_text(json.dumps({"moves": entries}))

    def report(self):
        return audit(self.api, self.pw, self.engine)

    def test_status_precedence_and_missing_manifest_file(self):
        self.api_moves([[1, "pound"], [2, "karate-chop"], [3, "double-slap"], [4, "comet-punch"]])
        self.local_move(3, "double-slap")
        self.lua("pound")
        self.lua("karate-chop")
        self.manifest([{"id": 3, "slug": "double-slap", "name": "Double Slap",
                        "file": "moves/double-slap.lua"}])
        report = self.report()
        self.assertEqual(report["counts"], {"MISSING": 1, "PRESENT": 2, "TO_BE_PORTED": 1})
        self.assertEqual({row["id"]: row["status"] for row in report["rows"]},
                         {1: "PRESENT", 2: "PRESENT", 3: "TO_BE_PORTED", 4: "MISSING"})

    def test_numeric_ids_join_renamed_slugs_and_struggle_is_present(self):
        self.api_moves([[1, "old-pokeapi-slug"], [165, "struggle"], [76, "solar-beam"]])
        self.lua("renamed-pound")
        self.lua("solar-beam")
        self.manifest([{"id": 1, "slug": "renamed-pound", "name": "Pound",
                        "file": "moves/renamed-pound.lua"}])
        system = self.engine / "crates/wilds-battle-engine/src/system_moves.lua"
        system.parent.mkdir(parents=True)
        system.write_text('return {id="struggle"}')
        report = self.report()
        self.assertEqual(report["counts"]["PRESENT"], 3)
        self.assertEqual(report["rows"][0]["slug"], "pound")
        self.assertEqual(report["rows"][-1]["engine_file"], str(system.relative_to(self.engine)))

    def test_scope_excludes_z_max_arceus_newer_and_shadow_but_keeps_ordinary_moves(self):
        for move_id in [622, 623, 658, 702, 719, 728]:
            self.assertEqual(exclusion_reason(move_id), "Z_MOVE")
        for move_id in [743, 757, 774]:
            self.assertEqual(exclusion_reason(move_id), "MAX_MOVE")
        for move_id in [827, 850]:
            self.assertEqual(exclusion_reason(move_id), "LEGENDS_ARCEUS")
        self.assertEqual(exclusion_reason(851), "AFTER_SWORD_SHIELD")
        self.assertEqual(exclusion_reason(10001), "SHADOW_MOVE")
        for move_id in [237, 251, 374, 718, 729, 744, 775, 781, 826]:
            self.assertEqual(exclusion_reason(move_id), "")
        self.api_moves([[1, "pound"], [743, "max-guard"], [827, "dire-claw"]])
        self.lua("pound")
        self.lua("max-guard")
        self.local_move(827, "dire-claw")
        report = self.report()
        self.assertEqual(report["counts"], {"MISSING": 0, "PRESENT": 1, "TO_BE_PORTED": 0})
        self.assertEqual(report["exclusions"], {"LEGENDS_ARCEUS": 1, "MAX_MOVE": 1})
        self.assertEqual(report["rows"][1]["engine_file"], "moves/max-guard.lua")

    def test_old_pokeapi_snapshot_does_not_drop_local_moves(self):
        self.api_moves([[1, "pound"]])
        self.lua("pound")
        self.local_move(2, "karate-chop")
        self.lua("double-slap")
        self.manifest([{"id": 3, "slug": "double-slap", "name": "Double Slap",
                        "file": "moves/double-slap.lua"}])
        report = self.report()
        self.assertEqual(report["counts"], {"MISSING": 0, "PRESENT": 2, "TO_BE_PORTED": 1})
        self.assertEqual([r["id"] for r in report["rows"] if not r["in_pokeapi"]], [2, 3])

    def test_unknown_lua_file_fails_instead_of_silently_undercounting(self):
        self.api_moves([[1, "pound"]])
        self.lua("unmapped")
        with self.assertRaisesRegex(ValueError, "Cannot map"):
            self.report()

    def test_duplicate_source_ids_and_mismatched_json_ids_fail(self):
        self.api_moves([[1, "pound"], [1, "duplicate"]])
        self.lua("pound")
        with self.assertRaisesRegex(ValueError, "Duplicate move ID"):
            self.report()
        self.api_moves([[1, "pound"]])
        (self.pw / "2.json").write_text('{"ID": 1, "Name": "pound", "HumanName": "Pound"}')
        with self.assertRaisesRegex(ValueError, "does not match filename"):
            self.report()


if __name__ == "__main__":
    unittest.main()
