"""Plot the north_up flag across two real GRIB sources of opposite native
orientation, as a 2x2 grid: {HRRR south-first, GEFS north-first} x {off, on}.

Each panel draws the decoded field with array row 0 at the top, so a correctly
north-up panel has its northern-most data at the top. The corner latitudes are
annotated so the orientation is unambiguous. Run:

    python examples/plot_north_up.py [output.png]
"""

import sys
from pathlib import Path

import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

import gribberish as g

TEST_DATA = Path(__file__).resolve().parents[2] / "test-data"

SOURCES = [
    ("HRRR TMP (Lambert, south-first)", "hrrr.t06z.wrfsfcf01-TMP.grib2"),
    ("GEFS HGT (global, north-first)", "geavg.t12z.pgrb2a.0p50.f000"),
]


def decode(fname, north_up):
    raw = (TEST_DATA / fname).read_bytes()
    meta = g.parse_grib_message_metadata(raw, 0)
    ny, nx = meta.grid_shape
    data = np.asarray(g.parse_grib_array(raw, 0, False, north_up)).reshape(ny, nx)
    lat, _ = meta.latlng(False, north_up)
    lat = np.asarray(lat)
    row_lat = (lat.reshape(ny, nx)[:, 0]) if lat.size == ny * nx else lat
    return data, row_lat


def main(out_path):
    fig, axes = plt.subplots(2, 2, figsize=(11, 9), constrained_layout=True)
    for row, (title, fname) in enumerate(SOURCES):
        for col, north_up in enumerate([False, True]):
            ax = axes[row][col]
            data, row_lat = decode(fname, north_up)
            # Row 0 at the top of the image; no y-flip, so the panel shows the
            # data exactly as row-ordered in memory.
            ax.imshow(data, origin="upper", aspect="auto", cmap="viridis")
            ax.set_title(f"{title}\nnorth_up={north_up}", fontsize=10)
            ax.set_xticks([])
            ax.set_yticks([])
            ax.set_ylabel(f"top row lat {row_lat[0]:.1f}", fontsize=8)
            ax.text(
                0.5, -0.06, f"bottom row lat {row_lat[-1]:.1f}",
                transform=ax.transAxes, ha="center", va="top", fontsize=8,
            )
    fig.suptitle("north_up: top row is northern-most when on", fontsize=13)
    fig.savefig(out_path, dpi=110)
    print(f"wrote {out_path}")


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "north_up_2x2.png"
    main(out)
