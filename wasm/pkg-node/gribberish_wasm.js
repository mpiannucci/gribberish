let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;
imports['env'] = require('env');
let wasm;
const { TextDecoder } = require(`util`);

const heap = new Array(32).fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function getObject(idx) { return heap[idx]; }

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}

let cachegetInt32Memory0 = null;
function getInt32Memory0() {
    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory0;
}

let cachegetFloat64Memory0 = null;
function getFloat64Memory0() {
    if (cachegetFloat64Memory0 === null || cachegetFloat64Memory0.buffer !== wasm.memory.buffer) {
        cachegetFloat64Memory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachegetFloat64Memory0;
}

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1);
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}
/**
* @param {Uint8Array} data
* @param {number} offset
* @returns {GribMessage}
*/
module.exports.parseGribMessage = function(data, offset) {
    var ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
    var len0 = WASM_VECTOR_LEN;
    var ret = wasm.parseGribMessage(ptr0, len0, offset);
    return GribMessage.__wrap(ret);
};

/**
* @param {Uint8Array} data
* @returns {Array<GribMessage>}
*/
module.exports.parseGribMessages = function(data) {
    var ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
    var len0 = WASM_VECTOR_LEN;
    var ret = wasm.parseGribMessages(ptr0, len0);
    return takeObject(ret);
};

/**
*/
class GribMessage {

    static __wrap(ptr) {
        const obj = Object.create(GribMessage.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gribmessage_free(ptr);
    }
    /**
    * @returns {string}
    */
    get varName() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.gribmessage_var_name(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(r0, r1);
        }
    }
    /**
    * @returns {string}
    */
    get varAbbrev() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.gribmessage_var_abbrev(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(r0, r1);
        }
    }
    /**
    * @returns {string}
    */
    get units() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.gribmessage_units(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(r0, r1);
        }
    }
    /**
    * @returns {number | undefined}
    */
    get arrayIndex() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.gribmessage_array_index(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return r0 === 0 ? undefined : r1 >>> 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @returns {Region}
    */
    get region() {
        var ret = wasm.gribmessage_region(this.ptr);
        return Region.__wrap(ret);
    }
    /**
    * @returns {GridShape}
    */
    get gridShape() {
        var ret = wasm.gribmessage_grid_shape(this.ptr);
        return GridShape.__wrap(ret);
    }
    /**
    * @returns {Date}
    */
    get forecastDate() {
        var ret = wasm.gribmessage_forecast_date(this.ptr);
        return takeObject(ret);
    }
    /**
    * @returns {Date}
    */
    get referenceDate() {
        var ret = wasm.gribmessage_reference_date(this.ptr);
        return takeObject(ret);
    }
    /**
    * @param {number} lat
    * @param {number} lon
    * @returns {number | undefined}
    */
    dataAtLocation(lat, lon) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.gribmessage_dataAtLocation(retptr, this.ptr, lat, lon);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getFloat64Memory0()[retptr / 8 + 1];
            return r0 === 0 ? undefined : r1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @returns {Float64Array}
    */
    data() {
        var ret = wasm.gribmessage_data(this.ptr);
        return takeObject(ret);
    }
    /**
    * @param {number} lat
    * @param {number} lon
    * @returns {number | undefined}
    */
    locationDataIndex(lat, lon) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.gribmessage_locationDataIndex(retptr, this.ptr, lat, lon);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return r0 === 0 ? undefined : r1 >>> 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
module.exports.GribMessage = GribMessage;
/**
*/
class GridShape {

    static __wrap(ptr) {
        const obj = Object.create(GridShape.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gridshape_free(ptr);
    }
    /**
    * @returns {number}
    */
    get rows() {
        var ret = wasm.__wbg_get_gridshape_rows(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} arg0
    */
    set rows(arg0) {
        wasm.__wbg_set_gridshape_rows(this.ptr, arg0);
    }
    /**
    * @returns {number}
    */
    get cols() {
        var ret = wasm.__wbg_get_gridshape_cols(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} arg0
    */
    set cols(arg0) {
        wasm.__wbg_set_gridshape_cols(this.ptr, arg0);
    }
}
module.exports.GridShape = GridShape;
/**
*/
class LatLon {

    static __wrap(ptr) {
        const obj = Object.create(LatLon.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_latlon_free(ptr);
    }
    /**
    * @returns {number}
    */
    get lat() {
        var ret = wasm.__wbg_get_latlon_lat(this.ptr);
        return ret;
    }
    /**
    * @param {number} arg0
    */
    set lat(arg0) {
        wasm.__wbg_set_latlon_lat(this.ptr, arg0);
    }
    /**
    * @returns {number}
    */
    get lon() {
        var ret = wasm.__wbg_get_latlon_lon(this.ptr);
        return ret;
    }
    /**
    * @param {number} arg0
    */
    set lon(arg0) {
        wasm.__wbg_set_latlon_lon(this.ptr, arg0);
    }
}
module.exports.LatLon = LatLon;
/**
*/
class Region {

    static __wrap(ptr) {
        const obj = Object.create(Region.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_region_free(ptr);
    }
    /**
    * @returns {LatLon}
    */
    get topLeft() {
        var ret = wasm.__wbg_get_region_topLeft(this.ptr);
        return LatLon.__wrap(ret);
    }
    /**
    * @param {LatLon} arg0
    */
    set topLeft(arg0) {
        _assertClass(arg0, LatLon);
        var ptr0 = arg0.ptr;
        arg0.ptr = 0;
        wasm.__wbg_set_region_topLeft(this.ptr, ptr0);
    }
    /**
    * @returns {LatLon}
    */
    get bottomRight() {
        var ret = wasm.__wbg_get_region_bottomRight(this.ptr);
        return LatLon.__wrap(ret);
    }
    /**
    * @param {LatLon} arg0
    */
    set bottomRight(arg0) {
        _assertClass(arg0, LatLon);
        var ptr0 = arg0.ptr;
        arg0.ptr = 0;
        wasm.__wbg_set_region_bottomRight(this.ptr, ptr0);
    }
}
module.exports.Region = Region;

module.exports.__wbindgen_number_new = function(arg0) {
    var ret = arg0;
    return addHeapObject(ret);
};

module.exports.__wbindgen_object_drop_ref = function(arg0) {
    takeObject(arg0);
};

module.exports.__wbg_gribmessage_new = function(arg0) {
    var ret = GribMessage.__wrap(arg0);
    return addHeapObject(ret);
};

module.exports.__wbg_new_ec75d0d5815be736 = function() {
    var ret = new Array();
    return addHeapObject(ret);
};

module.exports.__wbg_push_0daae9343162dbe7 = function(arg0, arg1) {
    var ret = getObject(arg0).push(getObject(arg1));
    return ret;
};

module.exports.__wbg_new_6bcf342e75fb6e1e = function(arg0) {
    var ret = new Date(getObject(arg0));
    return addHeapObject(ret);
};

module.exports.__wbg_buffer_79a3294266d4e783 = function(arg0) {
    var ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};

module.exports.__wbg_newwithbyteoffsetandlength_9d5c9051838c9a27 = function(arg0, arg1, arg2) {
    var ret = new Float64Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

module.exports.__wbg_new_6327eb6a637310e8 = function(arg0) {
    var ret = new Float64Array(getObject(arg0));
    return addHeapObject(ret);
};

module.exports.__wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

module.exports.__wbindgen_memory = function() {
    var ret = wasm.memory;
    return addHeapObject(ret);
};

const path = require('path').join(__dirname, 'gribberish_wasm_bg.wasm');
const bytes = require('fs').readFileSync(path);

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;
module.exports.__wasm = wasm;

