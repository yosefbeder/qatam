from subprocess import Popen, run
from os import listdir
from pathlib import Path
from threading import Timer
import json
from time import sleep


PATH = Path(__file__).parent.parent.joinpath('www')
PKG = json.loads(PATH.joinpath('package.json').read_text())


class Watcher:
    def __init__(self, paths):
        self.paths = paths
        self._cached_mtimes = self._get_mtimes()

    def did_change(self):
        mtimes = self._get_mtimes()
        if mtimes != self._cached_mtimes:
            self._cached_mtimes = mtimes
            return True
        return False

    def _get_mtimes(self):
        return list(map(lambda p: round(p.stat().st_mtime), self.paths))


def main():
    server_watcher = Watcher(list_paths_in(PATH, absolutize_paths(PATH, ['node_modules',
                                                                         'public', 'rollup.config.js']), '.js'))
    client_watcher = Watcher(list_paths_in(PATH, absolutize_paths(PATH, ['node_modules',
                                                                         'src', 'public/javascript/dist', 'rollup.config.js']), '.js'))

    bundle_js()
    server = run_server()

    while True:
        try:
            sleep(0.1)
            if client_watcher.did_change():
                bundle_js()
            if server_watcher.did_change():
                server.kill()
                server = run_server()
        except KeyboardInterrupt:
            break


def absolutize_paths(dir, paths):
    return list(map(lambda name: dir.joinpath(name), paths))


def list_paths_in(dir, ignored=[], ext=None):
    paths = []
    for file in listdir(dir):
        path = dir.joinpath(file)
        if path in ignored:
            continue

        if path.is_dir():
            def isChildPath(parent, suspect):
                return str(suspect).startswith(str(parent))

            for path in list_paths_in(path, list(filter(lambda suspect: isChildPath(path, suspect), ignored)), ext):
                paths.append(path)
            continue

        if ext is None:
            paths.append(path)
        elif path.suffix == ext:
            paths.append(path)
    return paths


def bundle_js():
    run(['npx.cmd', 'rollup', '-c'], cwd=PATH)


def run_server():
    return Popen(['node', PKG['main']], cwd=PATH)


main()
