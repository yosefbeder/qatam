import sys
import os
from pathlib import Path
import subprocess
from difflib import Differ

DEFAULT_DIR = Path(__file__).parent.parent.joinpath('tests')
BIN_DIR = Path(__file__).parent.parent.joinpath("target/release")


def serialize(returncode, stdout, stderr):
    return f"returncode: {returncode}\nstdout:\n{stdout}stderr:\n{stderr}"


def sync(dir):
    try:
        for name in os.listdir(dir):
            path = dir.joinpath(name)
            if path.is_dir():
                sync(path)
            if path.suffix == ".قتام":
                process = subprocess.run(
                    [BIN_DIR.joinpath("قتام.exe"), path], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, encoding="utf-8")
                snapshotPath = path.with_suffix(".النتيجة")
                open(snapshotPath, "w", encoding="utf-8").write(
                    serialize(process.returncode, process.stdout, process.stderr))
    except WindowsError as e:
        print(e)


def run(dir):
    try:
        for name in os.listdir(dir):
            path = dir.joinpath(name)
            if path.is_dir():
                run(path)
            if path.suffix == ".قتام":
                snapshotPath = path.with_suffix(".النتيجة")
                if not snapshotPath.exists():
                    raise RuntimeError(
                        f"The snapshot file {snapshotPath} does not exist\nhint: run the sync subcommand first")
                process = subprocess.run(
                    [BIN_DIR.joinpath("قتام.exe"), path], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, encoding="utf-8")
                res = serialize(process.returncode,
                                process.stdout, process.stderr)
                snapshot = open(snapshotPath, "r", encoding="utf-8").read(
                )
                if res != snapshot:
                    d = Differ()
                    diff = '\n'.join(
                        list(d.compare(res.splitlines(), snapshot.splitlines())))
                    print(
                        f"Error: {name} doesn't match its snapshot")
                    print(diff)
    except WindowsError as e:
        print(e)


def clean(dir):
    try:
        for name in os.listdir(dir):
            path = dir.joinpath(name)
            if path.is_dir():
                clean(path)
            if path.suffix == ".النتيجة":
                path.unlink()
    except WindowsError as e:
        print(e)


subcommands = {
    "sync": {"func": sync, "desc": f"Runs all of the files inside the dir speceified as an argument (or the default one which is {DEFAULT_DIR}) and creates snapshots."},
    "run": {"func": run, "desc": "Runs all of the files inside the dir and compares the result with the snapshots."},
    "clean": {"func": clean, "desc": "Cleans all of the snapshots in the passed dir"}
}


HELP_MSG = "USAGE: {} <subcommand> dir?\nAvaiable subcommands:\n{}".format(
    sys.argv[0],
    '\n'.join(map(lambda x: x['desc'], subcommands.values())))


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
            dir = Path(os.getcwd()).joinpath(next(argvIter))
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
