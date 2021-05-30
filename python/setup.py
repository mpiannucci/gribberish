from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="gribberish",
    version="0.9.0",
    rust_extensions=[RustExtension("gribberish.gribberish", binding=Binding.PyO3)],
    packages=["gribberish"],
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
)