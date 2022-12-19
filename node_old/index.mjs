import gribberish_rust from './index.node';

/**
 * Read a grib message from a string of byte data
 * @param {Buffer} data 
 * @param {number} offset 
 * @returns {GribMessage}
 */
function parseGribMessage(data, offset) {
    const gmHandle = gribberish_rust.parseGribMessage.call(data, offset);
    return new GribMessage(gmHandle);
}

/**
 * Read all of the grib messages in a string of byte data
 * @param {Uint8Array} data 
 * @returns {Array<GribMessage>}
 */
function parseGribMessages(data) {
    return gribberish_rust
        .parseGribMessages(data)
        .map(gmHandle => new GribMessage(gmHandle));
}

class GribMessage {
    /**
     * Creates a GribMessage JS wrapper around a rust GribMessage object
     * @param {*} handle rust object handle to grib message
     */
    constructor(handle) {
        this.gm = handle;
    }

    /**
     * Get the variable name of the data in the grib file
     * @returns {string}
     */
    get varName() {
        return gribberish_rust.gribMessageGetVarName.call(this.gm);
    }

    /**
     * Get the variable abbreviation of the data in the grib file
     * @returns {string}
     */
    get varAbbrev() {
        return gribberish_rust.gribMessageGetVarAbbrev.call(this.gm);
    }

    /**
     * Get the units of the grib message 
     * @returns {string}
     */
    get units() {
        return gribberish_rust.gribMessageGetUnits.call(this.gm);
    }

    /**
     * Get the array index if available. If not availble -1 is returned
     * @returns {number}
     */
    get arrayIndex() {
        return gribberish_rust.gribMessageGetArrayIndex.call(this.gm);
    }

    /**
     * Get the region of the grib message in lat,lng bounding box
     * @returns {{topLeft: {lat: number, lon: number}, bottomRight: {lat: number, lon: number}}}
     */
    get region() {
        return gribberish_rust.gribMessageGetRegion.call(this.gm);
    }

    /**
     * Get the shape of the data grid for the grib message
     * @returns {{rows: number, cols: number}}
     */
    get gridShape() {
        return gribberish_rust.gribMessageGetGridShape.call(this.gm);
    }

    /**
    * Get the forecast date for the grib message
    * @returns {Date}
    */
    get forecastDate() {
        return gribberish_rust.gribMessageGetForecastDate.call(this.gm);
    }

    /**
     * Get the reference date for the grib message
     * @returns {Date}
     */
    getReferenceDate() {
        return gribberish_rust.gribMessageGetReferenceDate.call(this.gm);
    }

    /**
     * Reads and returns the full data matrix from the grib message
     * data is lat*lon length and stored in row major order. It can be indexed with 
     * data[lat_index * (lon * lat_index) + lon_index]
     * @returns {Float64Array}
     */
    data() {
        const rawData = gribberish_rust.gribMessageGetData.call(this.gm);
        // ArrayBuffer is just a buffer of bytes... each Float64 item is 8 bytes long
        const data = new Float64Array(rawData, 0, rawData.byteLength / 8);
        return data;
    }

    /**
     * Get the data for a given latitude and longitude
     * @param {{lat: number, lon: number}} location 
     * @returns {number}
     */
    dataAtLocation(location) {
        return gribberish_rust.gribMessageGetDataAtLocation.call(this.gm, location.lat, location.lon);
    }

    /**
     * Get the data index for a given location 
     * @param {{lat: number, lon: number}} location 
     * @returns {number}
     */
    locationDataIndex(location) {
        return gribberish_rust.gribMessageGetLocationDataIndex.call(this.gm, location.lat, location.lon);
    }
}

export default {
    parseGribMessage,
    parseGribMessages,
    GribMessage,
};
