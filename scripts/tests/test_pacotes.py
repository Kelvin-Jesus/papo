"""Manifests for package managers come from the release checksums, never by hand.

Run: python3 -m unittest discover -s scripts/tests
"""

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

SPEC = importlib.util.spec_from_file_location(
    "pacotes", Path(__file__).resolve().parents[1] / "pacotes.py"
)
pacotes = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(pacotes)

TAG = "v0.3.0"
TARGETS = {
    "x86_64-unknown-linux-musl": "tar.gz",
    "aarch64-unknown-linux-musl": "tar.gz",
    "x86_64-apple-darwin": "tar.gz",
    "aarch64-apple-darwin": "tar.gz",
    "x86_64-pc-windows-msvc": "zip",
}


def sha_of(target: str) -> str:
    # Distinct, recognizable digest per target so a swapped checksum shows up.
    return (target.encode().hex() * 4)[:64]


def dist_dir() -> Path:
    d = Path(tempfile.mkdtemp())
    for target, ext in TARGETS.items():
        name = f"papo-{TAG}-{target}.{ext}"
        (d / f"{name}.sha256").write_text(f"{sha_of(target)}  {name}\n")
    return d


class Checksums(unittest.TestCase):
    def test_reads_every_target(self):
        sums = pacotes.ler_checksums(dist_dir(), TAG)
        self.assertEqual(set(sums), set(TARGETS))
        self.assertEqual(sums["aarch64-apple-darwin"], sha_of("aarch64-apple-darwin"))

    def test_missing_target_is_an_error(self):
        d = dist_dir()
        (d / f"papo-{TAG}-x86_64-pc-windows-msvc.zip.sha256").unlink()
        with self.assertRaises(SystemExit):
            pacotes.ler_checksums(d, TAG)


class Homebrew(unittest.TestCase):
    def setUp(self):
        self.rb = pacotes.formula_homebrew(TAG, pacotes.ler_checksums(dist_dir(), TAG))

    def test_version_without_v(self):
        self.assertIn('version "0.3.0"', self.rb)

    def test_each_platform_pairs_url_with_its_own_checksum(self):
        for target in ("x86_64-apple-darwin", "aarch64-apple-darwin",
                       "x86_64-unknown-linux-musl", "aarch64-unknown-linux-musl"):
            url = f"releases/download/{TAG}/papo-{TAG}-{target}.tar.gz"
            i = self.rb.index(url)
            self.assertEqual(self.rb[i:].split('sha256 "')[1][:64], sha_of(target), target)

    def test_installs_the_binary_from_the_archive_folder(self):
        self.assertIn('bin.install "papo"', self.rb)
        self.assertIn("papo --version", self.rb)


class Aur(unittest.TestCase):
    def setUp(self):
        sums = pacotes.ler_checksums(dist_dir(), TAG)
        self.pkgbuild = pacotes.pkgbuild_aur(TAG, sums)
        self.srcinfo = pacotes.srcinfo_aur(TAG, sums)

    def test_binary_package_conflicts_with_a_source_one(self):
        self.assertIn("pkgname=papo-bin", self.pkgbuild)
        self.assertIn("provides=('papo')", self.pkgbuild)
        self.assertIn("conflicts=('papo')", self.pkgbuild)

    def test_prebuilt_binary_is_not_stripped_nor_split_into_debug(self):
        self.assertIn("options=('!debug' '!strip')", self.pkgbuild)
        self.assertIn("options = !debug", self.srcinfo)

    def test_per_arch_sources_and_checksums(self):
        self.assertIn(f"sha256sums_x86_64=('{sha_of('x86_64-unknown-linux-musl')}')", self.pkgbuild)
        self.assertIn(f"sha256sums_aarch64=('{sha_of('aarch64-unknown-linux-musl')}')", self.pkgbuild)

    def test_srcinfo_matches_pkgbuild(self):
        self.assertIn("pkgbase = papo-bin", self.srcinfo)
        self.assertIn("pkgver = 0.3.0", self.srcinfo)
        self.assertIn(f"sha256sums_aarch64 = {sha_of('aarch64-unknown-linux-musl')}", self.srcinfo)


class Scoop(unittest.TestCase):
    def test_manifest_points_to_the_windows_zip(self):
        m = json.loads(pacotes.manifesto_scoop(TAG, pacotes.ler_checksums(dist_dir(), TAG)))
        x64 = m["architecture"]["64bit"]
        self.assertEqual(m["version"], "0.3.0")
        self.assertTrue(x64["url"].endswith(f"papo-{TAG}-x86_64-pc-windows-msvc.zip"))
        self.assertEqual(x64["hash"], sha_of("x86_64-pc-windows-msvc"))
        self.assertEqual(x64["extract_dir"], f"papo-{TAG}-x86_64-pc-windows-msvc")
        self.assertEqual(m["bin"], "papo.exe")


if __name__ == "__main__":
    unittest.main()
