import fs from 'fs';
import { parseMessagesFromBuffer } from 'gribberishjs';
import { createCanvas, Image } from 'canvas';
import { Resvg } from '@resvg/resvg-js';
import * as d3 from 'd3';

const gribData = fs.readFileSync('/Users/matthewiannucci/Downloads/MRMS_MergedReflectivityQCComposite_00.50_20230106-000439.grib2');
const gribMessages = parseMessagesFromBuffer(gribData);
const message = gribMessages.find(g => g.varAbbrev === 'MergedReflectivityQCComposite');

// const gribData = fs.readFileSync('./data/gfswave.20221222.t18z.atlocn.0p16.f064.grib2');
// const gribMessages = parseMessagesFromBuffer(gribData);
// const message = gribMessages.find(g => g.varAbbrev === 'HTSGW');

if (message !== undefined) {
  console.log('Found matching grib message, contouring...');
} else {
  console.error('Failed to find matching message. Exiting.');
  process.exit(0);
}

const bbox = message.bbox;
const lngRange = bbox[2] - bbox[0];
const latRange = bbox[3] - bbox[1];

const height = message.latitudes.length;
const width = message.longitudes.length;

const values = message.data;
for (let i = 0; i < values.length; ++i) {
  if (isNaN(values[i])) {
    values[i] = -99999;
  }
}

const max = d3.max(values);

// const blurredValues = d3.blur2({ data: swhMessage.data, width }, 0.5).data;
const contours = d3
  .contours()
  .size([width, height])
  .thresholds(Array.from({ length: 10 }, (_, i) => i / 10 * max));

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
fs.writeFileSync('vector.svg', svgout);

console.log('Rendering svg to image...');
const resvg = new Resvg(svgout)
const pngData = resvg.render()
const pngBuffer = pngData.asPng()

console.log('Writing to PNG file...');
fs.writeFileSync("./rendered.png", pngBuffer);

console.log('Operation Successful!');