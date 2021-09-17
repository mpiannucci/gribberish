import fs from 'fs';
import gribberish from 'gribberish';

const gribData = fs.readFileSync('../../../python/examples/gfswave.20210826.t12z.atlocn.0p16.f000.grib2');
const gribMessages = gribberish.parseGribMessages(gribData);

console.log(`Read ${gribMessages.length} grib messages`);
console.log(`Forecast Date: ${gribMessages[0].forecastDate.toLocaleString()}`);

gribMessages.forEach(gm => {
    //const data = gm.data();
    const fortyone_seventyone_data = gm.dataAtLocation({lat: 41.0, lon: 289.0});
    console.log(`${gm.varName} (${gm.varAbbrev}) - ${gm.units}: ${fortyone_seventyone_data}`);

    // if (gm.varName === 'HTSGW') {

    // }
});
