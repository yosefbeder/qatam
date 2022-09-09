import sys
from os import listdir
from shutil import rmtree
from pathlib import Path
import subprocess
from difflib import Differ
from datetime import datetime
from typing import Tuple

DEFAULT_DIR = Path("tests")
BIN_DIR = Path("target/release")


def build():
    subprocess.run(["cargo", "build", "--release"])


def serialize(returncode, stdout, stderr):
    return f"returncode: {returncode}\nstdout:\n{stdout}stderr:\n{stderr}"


def execute(path: Path) -> Tuple[str, str]:
    start = datetime.now()
    process = subprocess.run(
        [BIN_DIR.joinpath("قتام.exe"), path], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, encoding="utf-8")
    bench = {"path": path.as_posix(), "ms": round(
        (datetime.now() - start).total_seconds() * 1000)}
    return (serialize(process.returncode, process.stdout, process.stderr), bench)


def get_snapshot_path(dir: Path, name: str) -> Path:
    res = dir.joinpath("النتائج")
    if not res.is_dir():
        if res.is_file():
            res.unlink()
        res.mkdir()
    return res.joinpath(f"{name}.txt")


def sync(dir: Path, should_build: bool = True):
    if should_build:
        build()
    clean(dir)
    benches = []
    try:
        for name in listdir(dir):
            path = dir.joinpath(name)
            if path.is_dir():
                sync(path, False)
            if path.suffix == ".قتام":
                (res, bench) = execute(path)
                benches.append(bench)
                snapshot_path = get_snapshot_path(dir, name)
                open(snapshot_path, "w", encoding="utf-8").write(res)
    except WindowsError as e:
        print(e)
    for bench in benches:
        print("{:50} (in {}ms)".format(bench["path"], bench["ms"]))


def to_symbol(flag: bool) -> str:
    if flag:
        return "✓"
    else:
        return "✕"


def run(dir: Path, should_build: bool = True):
    if should_build:
        build()
    benches = []
    try:
        for name in listdir(dir):
            path = dir.joinpath(name)
            if path.is_dir() and name != "النتائج":
                run(path, False)
            if path.suffix == ".قتام":
                snapshot_path = get_snapshot_path(dir, name)
                if not snapshot_path.exists():
                    raise RuntimeError(
                        f"The snapshot file {snapshot_path} does not exist\nhint: run the sync subcommand first")
                (res, bench) = execute(path)
                snapshot = open(snapshot_path, "r", encoding="utf-8").read(
                )
                bench["succeeded"] = res == snapshot
                benches.append(bench)
    except WindowsError as e:
        print(e)
    for bench in benches:
        print("{:50} {} (in {}ms)".format(
            bench["path"], to_symbol(bench["succeeded"]), bench["ms"]))


def clean(dir: Path):
    try:
        for name in listdir(dir):
            path = dir.joinpath(name)
            if path.is_dir() and name == "النتائج":
                rmtree(path)
            elif path.is_dir():
                clean(path)
    except WindowsError as e:
        print(e)


subcommands = {
    "sync": {"func": sync, "desc": f"Runs all of the files inside the dir speceified as an argument (or the default one which is {DEFAULT_DIR}) and creates snapshots."},
    "run": {"func": run, "desc": "Runs all of the files inside the dir and compares the result with the snapshots."},
    "clean": {"func": clean, "desc": "Cleans all of the snapshots in the passed dir"}
}


HELP_MSG = "USAGE: {} <subcommand> dir?\nAvaiable subcommands:\n{}".format(
    sys.argv[0],
    '\n'.join(map(lambda x: f"{x[0]}: {x[1]['desc']}", subcommands.items())))


def main():
    try:
        argvIter = iter(sys.argv)
        next(argvIter)
        subcommand = next(argvIter)
    except StopIteration:
        print(HELP_MSG)
        return
    if subcommands.get(subcommand) is not None:
        try:
            dir = Path(next(argvIter))
        except StopIteration:
            dir = DEFAULT_DIR
        try:
            next(next(argvIter))
            assert False, "The sync and run subcommands take only one optional argument"
        except StopIteration:
            pass
        subcommands[subcommand]["func"](dir)
    else:
        print(
            f"Unknown subcommand: {subcommand}\n{HELP_MSG}")


main()
