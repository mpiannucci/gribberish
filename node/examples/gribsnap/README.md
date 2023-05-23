# gribsnap

Generate images from grib messages using `gribberish`, `d3`, and `node-canvas`

![example significant wave height output](./swh.png)

## Running

First install dependencies:

```bash
npm install
```

Then run the script:

```bash
# List variables
node index.mjs --path <path to grib file> --list

# Generate png
node index.mjs --path <path to grib file> --var <variable name> --png

# Generate svg
node index.mjs --path <path to grib file> --var <variable name> --svg

# Specify contouring
node index.mjs --path <path to grib file> --var <variable name> --png --minThreshold 0  --maxThreshold 100 --steps 10
```