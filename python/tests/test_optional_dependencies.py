import subprocess
import sys
from pathlib import Path

import pytest

pytest.importorskip("xarray")

from gribberish import gribberish_backend


def test_engine_discovery_without_obstore():
    # A fresh interpreter exercises entry-point loading without cached imports,
    # even when the test environment includes the xarray/virtualizarr extras.
    result = subprocess.run(
        [
            sys.executable,
            "-P",
            "-c",
            """
import importlib.abc
import sys
import warnings

class BlockObstore(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if fullname == "obstore" or fullname.startswith("obstore."):
            raise ModuleNotFoundError("No module named 'obstore'", name="obstore")

sys.meta_path.insert(0, BlockObstore())

import gribberish
gribberish.__path__.insert(0, sys.argv[1])
import xarray as xr

with warnings.catch_warnings():
    warnings.simplefilter("error")
    backend = xr.backends.list_engines()["gribberish"]
    assert backend.guess_can_open("example.grib2")
    assert not backend.guess_can_open("example.nc")

assert "gribberish._index" not in sys.modules
assert "obstore" not in sys.modules

for open_method in (backend.open_dataset, backend.open_groups_as_dict, backend.open_datatree):
    try:
        open_method("example.grib2")
    except ImportError as exc:
        assert 'pip install "gribberish[xarray]"' in str(exc), str(exc)
    else:
        raise AssertionError("Opening GRIB should require obstore")
""",
            str(Path(gribberish_backend.__file__).parent),
        ],
        capture_output=True,
        text=True,
        check=False,
        timeout=30,
    )
    assert result.returncode == 0, result.stdout + result.stderr


def test_obstore_internal_import_errors_propagate(monkeypatch):
    import builtins

    original_import = builtins.__import__
    error = ModuleNotFoundError(
        "Missing internal dependency", name="internal_dependency"
    )

    def import_with_broken_obstore(name, *args, **kwargs):
        if name == "obstore":
            raise error
        return original_import(name, *args, **kwargs)

    monkeypatch.setattr(builtins, "__import__", import_with_broken_obstore)
    with pytest.raises(ModuleNotFoundError) as caught:
        gribberish_backend._import_obstore()
    assert caught.value is error
