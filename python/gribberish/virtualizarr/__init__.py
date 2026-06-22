try:
    from .parser import GribberishParser
except ImportError as exc:  # pragma: no cover - exercised only when extra missing
    raise ImportError(
        "gribberish.virtualizarr requires the 'virtualizarr' extra. "
        "Install it with: pip install 'gribberish[virtualizarr]'"
    ) from exc

__all__ = ["GribberishParser"]
