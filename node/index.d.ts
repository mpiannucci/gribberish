/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface GridShape {
  rows: number
  cols: number
}
export function parseMessagesFromBuffer(buffer: Buffer): unknown[]
export class GribMessage {
  static parseFromBuffer(buffer: Buffer, offset: number): GribMessage
  get varName(): string
  get varAbbrev(): string
  get units(): string
  get arrayIndex(): number
  get forecastDate(): Date
  get referenceDate(): Date
  get proj(): string
  get crs(): string
  get bbox(): Array<number>
  get gridShape(): GridShape
  get gridResolution(): GridShape
  get latitudes(): Float64Array
  get longitudes(): Float64Array
  get data(): Float64Array
}