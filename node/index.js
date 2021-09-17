const gribberish_rust = require('./index.node');

/**
 * Read a grib message from a string of byte data
 * @param {Buffer} data 
 * @param {number} offset 
 * @returns {GribMessage}
 */
function parseGribMessage(data, offset) {
    const gmHandle =  gribberish_rust.parseGribMessage.call(data, offset);
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
    constructor(handle) {
        this.gm = handle;
    }

    /**
     * Get the variable name of the data in the grib file
     * @returns {string}
     */
    varName() {
        return gribberish_rust.gribMessageGetVarName.call(this.gm);
    }

    /**
     * Get the variable abbreviation of the data in the grib file
     * @returns {string}
     */
    varAbbrev() {
        return gribberish_rust.gribMessageGetVarAbbrev.call(this.gm);
    }

    /**
     * Get the units of the grib message 
     * @returns {string}
     */
    units() {
        return gribberish_rust.gribMessageGetUnits.call(this.gm);
    }
}

module.exports = {
    parseGribMessage, 
    parseGribMessages, 
    GribMessage,
};