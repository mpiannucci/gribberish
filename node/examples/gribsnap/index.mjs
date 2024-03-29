import fs from 'fs';
import minimist from 'minimist';
import { GribMessageFactory } from 'gribberishjs';
import { Resvg } from '@resvg/resvg-js';
import * as d3 from 'd3';

const args = minimist(process.argv.slice(2), {
  string: ['path', 'var', 'minThreshold', 'maxThreshold', 'steps', 'svgOut', 'pngOut', 'geojsonOut'],
  boolean: ['list', 'svg', 'png', 'geojson'],
  default: { vars: false, svg: false, png: true, geojson: false },
});

const gribPath = args.path;
const gribVariable = args.var

if (gribPath === undefined) {
  console.error('You must specify the path to the grib file to render with --path');
  process.exit(1);
}

const gribData = fs.readFileSync(gribPath);
const messageFactory = GribMessageFactory.fromBuffer(gribData);

if (args.list) {
  console.log(messageFactory.availableMessages);
  process.exit(0);
}

// WAVE
//const gribPath = './data/gfswave.20221222.t18z.atlocn.0p16.f064.grib2'
// const gribVariable = 'HTSGW@groundorwater_1'
// NWPS
// const gribPath = '/Users/matthewiannucci/Downloads/box_nwps_CG0_Trkng_20230109_0600.grib2';
// const gribVariable = 'SWPER@orderedsequence_1&2023-01-11T02:00:00+00:00';
// RADAR
// const gribPath = '/Users/matthewiannucci/Downloads/MRMS_MergedReflectivityQCComposite_00.50_20230106-000439.grib2'
// const gribVariable = 'MergedReflectivityQCComposite@specificaltitudeabovemeansealevel_500&2023-01-06T00:04:39+00:00'
// const gribPath = '/Users/matthewiannucci/Downloads/MRMS_PrecipRate_00.00_20230111-000600.grib2'
// const gribVariable = 'PrecipRate@specificaltitudeabovemeansealevel_0&2023-01-11T00:06:00+00:00'
// const gribPath = '/Users/matthewiannucci/Downloads/gfs.t18z.pgrb2.0p25.f186.grib2';
// const gribVariable = 'GUST@groundorwater_0';

const message = messageFactory.getMessage(gribVariable);

if (message !== undefined) {
  console.log('Found matching grib message, contouring...');
} else {
  console.error('Failed to find matching message. Exiting.');
  process.exit(0);
}

const bbox = message.bbox;

const { rows, cols } = message.gridShape;
const width = cols;
const height = rows;

const values = message.data;
const max = args.maxThreshold !== undefined ? parseFloat(args.maxThreshold) : d3.max(values);
const min = args.minThreshold !== undefined ? parseFloat(args.minThreshold) : d3.min(values);
const range = max - min;
const steps = args.steps !== undefined ? parseInt(args.steps) : 20;

console.log(`min: ${min}, max: ${max}, steps: ${steps}`);

for (let i = 0; i < values.length; ++i) {
  if (isNaN(values[i])) {
    values[i] = -9999999;
  }
}

// const blurredValues = d3.blur2({ data: swhMessage.data, width }, 0.5).data;
const contours = d3
  .contours()
  .size([width, height])
  .thresholds(Array.from({ length: steps }, (_, i) => min + (i / steps * range)));

// When geojson is specified, we map the coordinates and create a FeatureCollection to write out
if (args.geojson) {
  const lngRange = bbox[2] - bbox[0];
  const latRange = bbox[3] - bbox[1];

  const polygons = countours(data)
    .map(p => ({
      ...p,
      coordinates: p.coordinates
        .map(ring =>
          ring.map(coords =>
            coords
              .map(point => ([
                minLng + (lngRange) * (point[0] / width),
                maxLat + (latRange) * (point[1] / height)
              ]))
              .map(point => ([
                point[0] > 180 ? point[0] - 360 : point[0],
                point[1] > 90 ? point[1] - 90 : point[1],
              ]))
          )
        )
    }));

  const features = polygons.map(p => ({
    type: 'Feature',
    properties: {
      value: p.value,
      color: scale(p.value),
      variable,
      units,
    },
    geometry: {
      type: 'MultiPolygon',
      coordinates: p.coordinates,
    },
  }));

  const featureCollection = {
    type: 'FeatureCollection',
    features: features,
  };

  const geojsonOut = args.geojsonOut ?? `./${gribVariable}.json`;
  fs.writeFileSync(geojsonOut, JSON.stringify(featureCollection));
}

// When png or svg is enabled, we render specifically to the identity geo transform. 
if (args.png || args.svg) {
  const color = d3.scaleSequential([max, 0], d3.interpolateRdBu);

  // // For a different output projection, handle it with projection and scale 
  // const projection = d3.geoIdentity().scale(cols / cols);
  const path = d3.geoPath(d3.geoIdentity());

  console.log('Rendering contours...');
  const svgData = `
<svg style="width: 100%; height: auto; display: block;" viewBox="0 0 ${width} ${height}" xmlns="http://www.w3.org/2000/svg" xmlnsXlink="http://www.w3.org/1999/xlink'">
  ${Array.from(contours(values), d => `<path d="${path(d)}" fill="${color(d.value)}" />`).join('\n')}
</svg>
`;

  if (args.svg) {
    console.log('Writing to SVG file...');
    const svgOut = args.svgOut ?? `./${gribVariable}.svg`;
    fs.writeFileSync(svgOut, svgData);
  }

  if (args.png) {
    console.log('Rendering svg to image...');
    const resvg = new Resvg(svgData)
    const pngData = resvg.render()
    const pngBuffer = pngData.asPng()

    console.log('Writing to PNG file...');
    const pngOut = args.pngOut ?? `./${gribVariable}.png`;
    fs.writeFileSync(pngOut, pngBuffer);
  }
}

console.log('Operation Successful!');