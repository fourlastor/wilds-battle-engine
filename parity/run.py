#!/usr/bin/env python3
"""Compare a few live PokeWilds C# turns against the Rust engine."""

import os
import pathlib
import shutil
import subprocess
import sys
import tempfile


ROOT = pathlib.Path(__file__).resolve().parents[1]
CSHARP = ROOT / "parity" / "csharp"
SOURCE = ROOT.parent / "pokewilds-next"
ASSEMBLY = SOURCE / ".godot/mono/temp/bin/Debug/PokeWilds.dll"


def run(command: list[str], *, env: dict[str, str] | None = None) -> str:
    result = subprocess.run(
        command, cwd=ROOT, env=env, text=True, capture_output=True, check=False
    )
    if result.returncode or "ERROR:" in result.stdout or "ERROR:" in result.stderr:
        raise RuntimeError(
            f"{' '.join(command)} failed:\n{result.stdout}\n{result.stderr}"
        )
    return result.stdout


def fields(output: str, prefix: str, expected: set[str]) -> dict[str, str]:
    values = {}
    for key in expected:
        marker = f"{prefix}{key}="
        matches = [line.removeprefix(marker) for line in output.splitlines() if line.startswith(marker)]
        if len(matches) == 1:
            values[key] = matches[0]
    if set(values) != expected:
        raise RuntimeError(f"missing parity output:\n{output}")
    return values


def main() -> int:
    if not ASSEMBLY.is_file():
        raise RuntimeError(f"Build PokeWilds first; missing {ASSEMBLY}")
    sources = run(["git", "-C", str(SOURCE), "ls-files", "--", "*.cs"]).splitlines()
    newer = [
        SOURCE / name
        for name in sources
        if (SOURCE / name).stat().st_mtime > ASSEMBLY.stat().st_mtime
    ]
    if newer:
        raise RuntimeError(f"PokeWilds source is newer than {ASSEMBLY}; rebuild it first")

    run(
        [
            "dotnet", "build", str(CSHARP / "ParityHarness.csproj"),
            "--nologo", "-p:NuGetAudit=false",
        ]
    )
    with tempfile.TemporaryDirectory(prefix="wilds-parity-") as data_home:
        env = dict(os.environ, XDG_DATA_HOME=data_home)
        csharp_output = run(
            [
                shutil.which("godot") or "godot", "--headless", "--path",
                str(CSHARP), "--quit-after", "100",
            ],
            env=env,
        )
    for case_id, move_id in (
        ("dragon_rage", "dragon_rage"),
        ("dragon_rage_ko", "dragon_rage"),
        ("tackle_1", "tackle"),
        ("tackle_7", "tackle"),
        ("tackle_99", "tackle"),
        ("tackle_miss", "tackle"),
        ("tackle_crit", "tackle"),
        ("toxic", "toxic"),
        ("recover", "recover"),
    ):
        csharp = fields(
            csharp_output, f"PARITY_CSHARP_{case_id}_",
            {"EVENTS", "HP", "STATUS", "CHANCES", "ROLL"},
        )
        rust = fields(
            run(
                [
                    "cargo", "run", "-p", "wilds-battle-engine", "--example",
                    "parity_turn", "--offline", "--quiet", "--", case_id, move_id,
                    csharp["CHANCES"], csharp["ROLL"],
                ]
            ),
            f"PARITY_RUST_{case_id}_",
            {"EVENTS", "HP", "STATUS"},
        )
        expected = {key: csharp[key] for key in ("EVENTS", "HP", "STATUS")}
        if expected != rust:
            print(f"{case_id}\nC#:   {expected}\nRust: {rust}", file=sys.stderr)
            return 1
        print(f"{case_id}: C# and Rust match: {rust['EVENTS']} (HP {rust['HP']})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
