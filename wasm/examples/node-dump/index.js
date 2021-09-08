import fs from 'fs';
import gribberish from 'gribberish-wasm';

const gribData = fs.readFileSync('../../../python/examples/gfswave.20210826.t12z.atlocn.0p16.f000.grib2');

const gribMessages = gribberish.parseGribMessages(gribData);

console.log(`Read ${gribMessages.length} grib messages`);

gribMessages.forEach(gm => {
    console.log(`${gm.varName()} (${gm.varAbbrev()}) - ${gm.units()}`);
});
