from setuptools import setup
from setuptools_rust import RustExtension

setup(
    name="gribberish",
    version="0.9.0",
    rust_extensions=[RustExtension("gribberish.gribberish", "Cargo.toml")],
    packages=["gribberish"],
    include_package_data=True,
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
)
