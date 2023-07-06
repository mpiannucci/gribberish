from setuptools import setup
from setuptools_rust import RustExtension

setup(
    name="gribberish",
    version="0.10.0",
    rust_extensions=[RustExtension("gribberish.gribberishpy", debug=False)],
    packages=["gribberish"],
    include_package_data=True,
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
    requires=["numpy"],
    extras_require={
        "xarray": ["xarray"],
        "kerchunk": ["kerchunk", "zarr", "numcodecs", "fsspec"],
    },
    entry_points={
      "xarray.backends": ["gribberish=gribberish.gribberish_backend:GribberishBackend"],
    },
)
