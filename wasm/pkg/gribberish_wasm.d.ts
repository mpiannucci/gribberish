/* tslint:disable */
/* eslint-disable */
/**
* @param {Uint8Array} data
* @param {number} offset
* @returns {GribMessage}
*/
export function parse_grib_message(data: Uint8Array, offset: number): GribMessage;
/**
* @param {Uint8Array} data
* @returns {Array<GribMessage>}
*/
export function pase_grib_messages(data: Uint8Array): Array<GribMessage>;
/**
*/
export class GribMessage {
  free(): void;
/**
* @returns {string}
*/
  var_name(): string;
/**
* @returns {string}
*/
  var_abbrev(): string;
/**
* @returns {string}
*/
  units(): string;
}
