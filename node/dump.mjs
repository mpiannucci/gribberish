import fs from 'fs';
import { parseMessagesFromBuffer } from './index.js';

const gribData = fs.readFileSync('../python/examples/gfswave.20210826.t12z.atlocn.0p16.f000.grib2');
const gribMessages = parseMessagesFromBuffer(gribData);

console.log(`Read ${gribMessages.length} grib messages`);

gribMessages.forEach((gm, i) => {
    console.log(`${i}: ${gm.varName}, (${gm.varAbbrev}), ${gm.units}, bbox='${gm.bbox}'`);
});
