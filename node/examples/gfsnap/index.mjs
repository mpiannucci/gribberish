import fs from 'fs';
import { GribMessageFactory } from 'gribberishjs';
import { Resvg } from '@resvg/resvg-js';
import * as d3 from 'd3';

// WAVE
//const gribPath = './data/gfswave.20221222.t18z.atlocn.0p16.f064.grib2'
// const gribVariable = 'HTSGW@groundorwater_1'
// NWPS
const gribPath = '/Users/matthewiannucci/Downloads/box_nwps_CG0_Trkng_20230109_0600.grib2';
const gribVariable = 'SWPER@orderedsequence_1&2023-01-11T02:00:00+00:00';
// RADAR
// const gribPath = '/Users/matthewiannucci/Downloads/MRMS_MergedReflectivityQCComposite_00.50_20230106-000439.grib2'
// const gribVariable = 'MergedReflectivityQCComposite'
// const gribPath = '/Users/matthewiannucci/Downloads/gfs.t18z.pgrb2.0p25.f186.grib2';
// const gribVariable = 'GUST@groundorwater_0';

const gribData = fs.readFileSync(gribPath);
const messageFactory = GribMessageFactory.fromBuffer(gribData);

// console.log(messageFactory.availableMessages);
// process.exit(0);

const message = messageFactory.getMessage(gribVariable);

if (message !== undefined) {
  console.log('Found matching grib message, contouring...');
} else {
  console.error('Failed to find matching message. Exiting.');
  process.exit(0);
}

const bbox = message.bbox;
const lngRange = bbox[2] - bbox[0];
const latRange = bbox[3] - bbox[1];

console.log(bbox);
console.log(lngRange);
console.log(latRange);

const height = message.latitudes.length;
const width = message.longitudes.length;

const values = message.data;
const max = d3.max(values);
const min = d3.min(values);
const range = max - min;
const steps = 20;

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

const color = d3.scaleSequential([max, 0], d3.interpolateRdBu);

// const canvas = createCanvas(width, height)
// const context = canvas.getContext("2d");

// context.lineWidth = 2;
// context.lineJoin = "round";
// context.strokeStyle = "black"
//   ;
// // For a different output projection, handle it with projection and scale 
// const projection = d3.geoIdentity().scale(cols / cols);
const path = d3.geoPath(d3.geoIdentity());

// context.beginPath();
// geoPath(contours(data));
// context.stroke();

// canvas.createPNGStream().pipe(fs.createWriteStream("./swh.png"));

// const polys = contours(blurredValues);

console.log('Rendering to SVG...');
const svgout = `
<svg style="width: 100%; height: auto; display: block;" viewBox="0 0 ${width} ${height}" xmlns="http://www.w3.org/2000/svg" xmlnsXlink="http://www.w3.org/1999/xlink'">
  ${Array.from(contours(values), d => `<path d="${path(d)}" fill="${color(d.value)}" />`).join('\n')}
</svg>
`;

console.log('Writing to SVG file...');
fs.writeFileSync(`${gribVariable}.svg`, svgout);

console.log('Rendering svg to image...');
const resvg = new Resvg(svgout)
const pngData = resvg.render()
const pngBuffer = pngData.asPng()

console.log('Writing to PNG file...');
fs.writeFileSync(`./${gribVariable}.png`, pngBuffer);

console.log('Operation Successful!');