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
* @returns {string}
*/
  varName(): string;
/**
* @returns {string}
*/
  varAbbrev(): string;
/**
* @returns {string}
*/
  units(): string;
}
