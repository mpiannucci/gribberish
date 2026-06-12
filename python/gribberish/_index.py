"""Shared helpers for GRIB sidecar index files (NOAA wgrib2 ``.idx`` and
ECMWF open-data ``.index``), used by the xarray backend and the VirtualiZarr
parser to locate messages without downloading the whole GRIB file."""

import asyncio
from concurrent.futures import ThreadPoolExecutor

import obstore

from gribberish import parse_grib_index

# Plenty for sections 0-5 of a message, which is all the metadata needs; the
# data section (and any large bitmap) is never fetched. If a message's header
# sections exceed this, parsing fails cleanly and callers fetch the whole
# message instead.
HEADER_BYTES = 4096


# Ranges per get_ranges_async call. obstore caps each call at 10 parallel
# fetches; gathering many calls lifts the cap (measured ~10x faster headers
# on a 743-message GFS file: 6s -> 0.5s).
_RANGES_PER_CALL = 25


def _run_async(coro):
    """Run a coroutine from sync code, even when a loop is already running
    (e.g. Jupyter) — same bridge VirtualiZarr's icechunk parser uses."""
    try:
        asyncio.get_running_loop()
    except RuntimeError:
        return asyncio.run(coro)
    with ThreadPoolExecutor(1) as executor:
        return executor.submit(asyncio.run, coro).result()


def get_ranges_batched(store, path, starts, lengths, coalesce):
    """Fetch byte ranges with real concurrency: many get_ranges_async calls
    gathered at once, each internally fetching up to 10 ranges in parallel.
    Pass a small ``coalesce`` when the gaps between ranges are bytes being
    deliberately skipped — obstore's 1MB default would transfer them."""

    async def gather():
        async def one(s, l):
            return await obstore.get_ranges_async(
                store, path, starts=s, lengths=l, coalesce=coalesce
            )

        tasks = [
            one(starts[i : i + _RANGES_PER_CALL], lengths[i : i + _RANGES_PER_CALL])
            for i in range(0, len(starts), _RANGES_PER_CALL)
        ]
        return [chunk for chunks in await asyncio.gather(*tasks) for chunk in chunks]

    return _run_async(gather())


def index_candidates(path):
    """Sidecar index names to probe: NOAA appends `.idx` to the full file
    name; ECMWF open data replaces the extension with `.index`."""
    stem, dot, _ = path.rpartition(".")
    candidates = [f"{path}.idx"]
    if dot:
        candidates.append(f"{stem}.index")
    candidates.append(f"{path}.index")
    return candidates


def fetch_index_entries(store, path):
    """Fetch and parse the sidecar index for ``path``, trying each candidate
    name. Raises FileNotFoundError when no index exists."""
    for candidate in index_candidates(path):
        try:
            text = obstore.get(store, candidate).bytes().to_bytes().decode()
        except FileNotFoundError:
            continue
        file_size = obstore.head(store, path)["size"]
        return parse_grib_index(text, file_size=file_size)
    raise FileNotFoundError(
        f"No index file found for {path!r} (tried {index_candidates(path)})"
    )


def select_ranges(entries, only_variables, drop_variables):
    """Byte ranges (offset -> length) of the messages the variable filters
    might keep. Pushdown is conservative: a message is skipped only when its
    index entry proves the filters drop it; the full filters still run over
    the fetched messages. Submessage entries share one byte range."""
    only = {v.lower() for v in only_variables} if only_variables else None
    drop = {v.lower() for v in drop_variables} if drop_variables else set()
    ranges = {}
    for entry in entries:
        var = entry.var.lower() if entry.var is not None else None
        if var is not None and (var in drop or (only is not None and var not in only)):
            continue
        ranges[entry.offset] = entry.length
    if not ranges:
        raise ValueError("No variables remain after filtering")
    return ranges
