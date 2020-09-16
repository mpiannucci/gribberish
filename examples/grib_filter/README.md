# Grib Filter

This example downloads data from [NOAA's Grib Filter](https://nomads.ncep.noaa.gov/txt_descriptions/grib_filter_doc.shtml) and outputs the data to a CSV. For now, it is sort of hardcoded for the multiwave model but it can be adapted to NWW3 and GFS pretty easily. 

Run this app from the command line in the projects root directory with: 

```bash
cargo run --example grib_filter
```

Once the app has finished, you can plot the data using `python3` and `matplotlib`

```bash
cd examples/grib_filter
pip install matplotlib
python plot_data.py
```