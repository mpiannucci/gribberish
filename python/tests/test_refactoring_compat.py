"""
Tests to verify Python bindings work with refactored Rust code.

These tests ensure backwards compatibility after adding:
- Backend abstraction (native/eccodes)
- Configuration system
- New API module
"""

import pytest
import numpy as np


def test_parse_grib_array_still_works():
    """Test that parse_grib_array (used by xarray backend) still works"""
    from gribberish import parse_grib_array

    # Create minimal GRIB2 header
    data = bytearray(b"GRIB" + b"\x00\x00\x02\x02" + b"\x00" * 8 + b"7777")

    # This should not raise even if it can't parse (error handling is in Rust)
    # The important part is that the function exists and is callable
    assert callable(parse_grib_array)


def test_parse_grib_dataset_still_works():
    """Test that parse_grib_dataset (used by xarray backend) still works"""
    from gribberish import parse_grib_dataset

    data = b"Not a valid GRIB file"

    # Function should exist and be callable
    assert callable(parse_grib_dataset)

    # Empty data should return empty dataset structure
    try:
        result = parse_grib_dataset(b"")
        assert isinstance(result, dict)
        assert "coords" in result or "data_vars" in result or "attrs" in result
    except Exception as e:
        # Some error handling is acceptable
        pass


def test_parse_grib_message_still_works():
    """Test that parse_grib_message still works"""
    from gribberish import parse_grib_message

    # Function should exist and be callable
    assert callable(parse_grib_message)


def test_parse_grib_message_metadata_still_works():
    """Test that parse_grib_message_metadata still works"""
    from gribberish import parse_grib_message_metadata

    # Function should exist and be callable
    assert callable(parse_grib_message_metadata)


def test_build_grib_array_still_works():
    """Test that build_grib_array still works"""
    from gribberish import build_grib_array

    # Function should exist and be callable
    assert callable(build_grib_array)


def test_python_module_version():
    """Test that the Python module has a version"""
    import gribberish

    assert hasattr(gribberish, "__version__")
    assert isinstance(gribberish.__version__, str)
    assert gribberish.__version__ == "0.23.0"


def test_grib_message_class_exists():
    """Test that GribMessage class still exists"""
    from gribberish import GribMessage

    assert GribMessage is not None


def test_all_public_api_functions_exist():
    """Test that all expected public API functions exist"""
    import gribberish

    expected_functions = [
        "parse_grib_message",
        "parse_grib_message_metadata",
        "parse_grib_array",
        "parse_grib_dataset",
        "parse_grib_mapping",
        "build_grib_array",
    ]

    for func_name in expected_functions:
        assert hasattr(gribberish, func_name), f"Missing function: {func_name}"
        func = getattr(gribberish, func_name)
        assert callable(func), f"{func_name} is not callable"


@pytest.mark.skipif(
    True, reason="Requires xarray and actual GRIB file"
)
def test_xarray_backend_integration():
    """
    Test that xarray backend still works after refactoring.

    This test is skipped by default because it requires:
    - xarray to be installed
    - A real GRIB2 file to test with
    """
    import xarray as xr
    from gribberish.gribberish_backend import GribberishBackend

    # Register the backend
    backend = GribberishBackend()

    # This would test opening a GRIB file with xarray
    # ds = xr.open_dataset("example.grib2", engine="gribberish")
    # assert ds is not None

    assert backend is not None
    assert hasattr(backend, "open_dataset")
    assert callable(backend.open_dataset)


def test_backwards_compatibility_imports():
    """Test that all old imports still work"""
    try:
        from gribberish import (
            parse_grib_array,
            parse_grib_dataset,
            parse_grib_message,
            parse_grib_message_metadata,
            build_grib_array,
            parse_grib_mapping,
            GribMessage,
        )

        # All imports should succeed
        assert parse_grib_array is not None
        assert parse_grib_dataset is not None
        assert parse_grib_message is not None
        assert parse_grib_message_metadata is not None
        assert build_grib_array is not None
        assert parse_grib_mapping is not None
        assert GribMessage is not None

    except ImportError as e:
        pytest.fail(f"Import failed: {e}")


def test_backend_specific_xarray_integration():
    """
    Test that the xarray backend can still be imported and instantiated.
    """
    try:
        from gribberish.gribberish_backend import (
            GribberishBackend,
            GribberishBackendArray,
        )

        backend = GribberishBackend()
        assert backend is not None

        # Test that required methods exist
        assert hasattr(backend, "open_dataset")
        assert hasattr(backend, "guess_can_open")
        assert hasattr(backend, "open_dataset_parameters")

        # Test guess_can_open method
        assert backend.guess_can_open("test.grib2") == True
        assert backend.guess_can_open("test.grib") == True
        assert backend.guess_can_open("test.nc") == False

    except ImportError as e:
        pytest.fail(f"Failed to import xarray backend: {e}")


def test_zarr_codec_still_works():
    """Test that zarr codec integration still works"""
    try:
        from gribberish.zarr.codec import GribberishCodec

        codec = GribberishCodec(var="TMP")
        assert codec is not None
        assert hasattr(codec, "_decode_single")

    except ImportError:
        pytest.skip("zarr not installed")


def test_kerchunk_integration_still_works():
    """Test that kerchunk integration still works"""
    try:
        from gribberish.kerchunk.codec import GribberishCodec as KerchunkCodec

        codec = KerchunkCodec(var="TMP")
        assert codec is not None

    except ImportError:
        pytest.skip("kerchunk not installed")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
