"""Run the packager against isolated fixtures; no real workspace files are changed."""
from pathlib import Path
import json
import subprocess
import sys
import tempfile
import unittest
import zipfile


PACKAGER = Path(__file__).with_name('package-v2.py')


class PackagingReportsTest(unittest.TestCase):
    def package_fixture(self, existing_report):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / 'workspace'
            root.mkdir()
            required = [
                'index.html', 'package.json', 'package-lock.json', 'vite.config.js',
                'svelte.config.js', 'start.cmd', 'start.ps1', '一键启动.cmd', '开启局域网访问.cmd', 'enable-team-lan.ps1', 'README.md', 'PRODUCT.md',
                'DESIGN.md', 'backend/Cargo.toml', 'backend/Cargo.lock', 'bin/edge-studio.exe',
                'src/main.js', 'dist/index.html', 'backend/src/main.rs',
            ]
            for name in required:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(('fixture: ' + name).encode())
            reports = root / 'docs/verification'
            reports.mkdir(parents=True)
            browser = reports / 'browser-report.json'
            # Include a failed check: packaging must not turn recorded failures into passes.
            evidence = b'{"run":"actual-fixture-run","browserChecks":[{"passed":false}]}'
            if existing_report:
                browser.write_bytes(evidence)
            (reports / 'p2-release.json').write_bytes(b'{"fixture":"retained P1/P2 evidence"}')
            private = root / 'backend/studio-data/project.json'
            private.parent.mkdir(parents=True)
            private.write_bytes(b'fixture user data, must not be packaged')
            team = root / 'backend/team-private/accounts.json'
            team.parent.mkdir(parents=True)
            team.write_bytes(b'private password hashes')
            before = {p.relative_to(root): p.read_bytes() for p in root.rglob('*') if p.is_file()}

            process = subprocess.run([sys.executable, str(PACKAGER)], cwd=root, capture_output=True, text=True)
            self.assertEqual(process.returncode, 0, process.stderr)
            result = json.loads(process.stdout)
            archive = root.parent / 'GVPAK-STUDIO.zip'
            self.assertEqual(result['archiveBytes'], archive.stat().st_size)
            after = {p.relative_to(root): p.read_bytes() for p in root.rglob('*') if p.is_file()}
            self.assertEqual(before, after, 'packaging must not create or overwrite source/evidence files')
            with zipfile.ZipFile(archive) as zipped:
                self.assertIsNone(zipped.testzip())
                self.assertEqual(result['files'], len(zipped.namelist()))
                self.assertEqual('docs/verification/browser-report.json' in zipped.namelist(), existing_report)
                if existing_report:
                    self.assertEqual(zipped.read('docs/verification/browser-report.json'), evidence)
                self.assertEqual(zipped.read('docs/verification/p2-release.json'), before[Path('docs/verification/p2-release.json')])
                for name in ['src/main.js', 'backend/src/main.rs', 'dist/index.html', 'bin/edge-studio.exe']:
                    self.assertEqual(zipped.read(name), before[Path(name)])
                self.assertNotIn('backend/studio-data/project.json', zipped.namelist())
                self.assertNotIn('backend/team-private/accounts.json', zipped.namelist())

    def test_existing_report_is_preserved_verbatim(self):
        self.package_fixture(existing_report=True)

    def test_missing_report_is_not_fabricated(self):
        self.package_fixture(existing_report=False)


if __name__ == '__main__':
    unittest.main()
