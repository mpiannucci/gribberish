import fs from 'fs';
import { parseMessagesFromBuffer } from 'gribberishjs';
import { createCanvas, Image } from 'canvas';
import * as d3 from 'd3';

const gribData = fs.readFileSync('./data/gfswave.20221222.t18z.atlocn.0p16.f064.grib2');
const gribMessages = parseMessagesFromBuffer(gribData);

const swhMessage = gribMessages.find(g => g.varAbbrev === 'HTSGW');

if (swhMessage !== undefined) {
  console.log('Found grib message for significant wave height, contouring...');
} else {
  console.error('Failed to find significant wave height message. Exiting.');
  process.exit(0);
}

console.log(swhMessage.bbox)

const bbox = swhMessage.bbox;
const lngRange = bbox[2] - bbox[0];
const latRange = bbox[3] - bbox[1];

console.log(lngRange);
console.log(latRange);

const height = swhMessage.latitudes.length;
const width = swhMessage.longitudes.length;

const values = swhMessage.data;
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
  .thresholds(Array.from({ length: 50 }, (_, i) => i / 50 * max));

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

const svgout = `
<svg style="width: 100%; height: auto; display: block;" viewBox="0 0 ${width} ${height}" xmlns="http://www.w3.org/2000/svg" xmlnsXlink="http://www.w3.org/1999/xlink'">
  ${Array.from(contours(values), d => `<path d="${path(d)}" fill="${color(d.value)}" />`).join('\n')}
</svg>
`;

fs.writeFileSync('swh.svg', svgout);

const canvas = createCanvas(width, height)
const ctx = canvas.getContext("2d");
ctx.fillStyle = '#FFFFFF';
ctx.fillRect(0, 0, width, height);

// Draw the svg image to the canvas
const img = new Image();
img.src = "data:image/svg+xml," + svgout;
ctx.drawImage(img, 0, 0, width, height);

canvas.createPNGStream().pipe(fs.createWriteStream("./swh.png"));