/* tslint:disable */
/* eslint-disable */
/**
* @param {Uint8Array} data
* @param {number} offset
* @returns {GribMessage}
*/
export function parseGribMessage(data: Uint8Array, offset: number): GribMessage;
/**
* @param {Uint8Array} data
* @returns {Array<GribMessage>}
*/
export function parseGribMessages(data: Uint8Array): Array<GribMessage>;
/**
*/
export class GribMessage {
  free(): void;
/**
* @param {number} lat
* @param {number} lon
* @returns {number | undefined}
*/
  dataAtLocation(lat: number, lon: number): number | undefined;
/**
* @returns {Float64Array}
*/
  data(): Float64Array;
/**
* @param {number} lat
* @param {number} lon
* @returns {number | undefined}
*/
  locationDataIndex(lat: number, lon: number): number | undefined;
/**
* @returns {number | undefined}
*/
  readonly arrayIndex: number | undefined;
/**
* @returns {Date}
*/
  readonly forecastDate: Date;
/**
* @returns {GridShape}
*/
  readonly gridShape: GridShape;
/**
* @returns {Date}
*/
  readonly referenceDate: Date;
/**
* @returns {Region}
*/
  readonly region: Region;
/**
* @returns {string}
*/
  readonly units: string;
/**
* @returns {string}
*/
  readonly varAbbrev: string;
/**
* @returns {string}
*/
  readonly varName: string;
}
/**
*/
export class GridShape {
  free(): void;
/**
* @returns {number}
*/
  cols: number;
/**
* @returns {number}
*/
  rows: number;
}
/**
*/
export class LatLon {
  free(): void;
/**
* @returns {number}
*/
  lat: number;
/**
* @returns {number}
*/
  lon: number;
}
/**
*/
export class Region {
  free(): void;
/**
* @returns {LatLon}
*/
  bottomRight: LatLon;
/**
* @returns {LatLon}
*/
  topLeft: LatLon;
}
