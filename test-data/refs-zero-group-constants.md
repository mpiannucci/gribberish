# REFS zero-group constant fixtures

These single-message fixtures come from the public NOAA RRFS bucket. Both use DRT 5.3 with zero groups, zero spatial-descriptor octets, missing-value management 0, and no bitmap. The grid is 1059 × 1799 (1,905,141 values), and Section 7 has no packed payload.

| Fixture | Source | Message | Byte range (inclusive) | Length | GDAL value at every cell |
| --- | --- | --- | --- | --- | --- |
| `20260915-12-prob-f01-22.grib2` | [Probability file](https://noaa-rrfs-ops-pds.s3.amazonaws.com/refs.20260915/12/ensprod/refs.t12z.prob.f01.conus.grib2) | 22 | 4204057–4204285 | 229 | 100.00001525878906 |
| `20260915-12-sprd-f01-63.grib2` | [Spread file](https://noaa-rrfs-ops-pds.s3.amazonaws.com/refs.20260915/12/ensprod/refs.t12z.sprd.f01.conus.grib2) | 63 | 61047683–61047900 | 218 | 0 |

The probability message represents convective inhibition below 0 J/kg in the 90–0 mb layer above ground. The spread message represents surface categorical rain. Neither has nodata cells.

SHA-256:

```text
b9d40a689f5ab0a99c2db48109da198d9ead1f50d8bf1c735955f369b8d25b1f  20260915-12-prob-f01-22.grib2
ba807b00bda3cf0abe60223ddfc4476a3b764dcefa07bdbefb86adcc8384cfc4  20260915-12-sprd-f01-63.grib2
```

The synthetic tests also exercise DRT 5.2, first- and second-order DRT 5.3, nonzero decimal scales, encoded-value counts, and mixed-bitmap expansion. Their unscaled-reference expectation follows the constant-field branches of NCEP's [g2c encoder](https://github.com/NOAA-EMC/NCEPLIBS-g2c/blob/ccc033cdd3997f41cec5d549689fc800944ead2b/src/compack.c#L392-L420) and [Fortran g2 encoder](https://github.com/NOAA-EMC/NCEPLIBS-g2/blob/87f8a69674c0d741009269aa4a07018257583ea7/src/compack.F90#L461-L482). These branches store the field value as the reference without applying the decimal scale. The [g2c decoder](https://github.com/NOAA-EMC/NCEPLIBS-g2c/blob/ccc033cdd3997f41cec5d549689fc800944ead2b/src/comunpack.c#L55-L72) and [ecCodes decoder](https://github.com/ecmwf/eccodes/blob/161bfaf69dc3627ee1e08b0771a0ab82f3fbec10/src/eccodes/accessor/DataG22OrderPacking.cc#L1587-L1593) return that reference unchanged. GDAL bundles g2c, so its agreement is not an independent scaling algorithm.

wgrib2's [default g2clib-emulation mode](https://github.com/NOAA-EMC/wgrib2/blob/46676e16bcb800044da412abe1135927758aed24/src/wgrib2.h#L74) also returns the unscaled reference: its [caller temporarily clears the decimal scale](https://github.com/NOAA-EMC/wgrib2/blob/46676e16bcb800044da412abe1135927758aed24/src/wgrib2.c#L746-L771). With `-g2clib 0`, its [internal decoder](https://github.com/NOAA-EMC/wgrib2/blob/46676e16bcb800044da412abe1135927758aed24/src/unpk_complex.c#L62-L94) instead applies the decimal scale. The modes differ when both the reference and decimal scale are nonzero. Both real fixtures have decimal scale zero; only the synthetic cases distinguish these conventions.

The direct-unpack matrix pins the chosen decoder convention, including synthetic count-zero and missing-management cases; it does not establish producer output for every header combination. Binary scale multiplies the packed increment, not the reference, and cannot distinguish the decimal conventions. The GDAL comparison covers the message-level synthetic cases, not every direct-unpack combination.
