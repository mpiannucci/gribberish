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
* @returns {GribMessages}
*/
export function pase_grib_messages(data: Uint8Array): GribMessages;
/**
*/
export class GribMessage {
  free(): void;
}
/**
*/
export class GribMessages {
  free(): void;
}
