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

The synthetic tests also exercise DRT 5.2, first- and second-order DRT 5.3, nonzero decimal scales, encoded-value counts, and mixed-bitmap expansion. For zero groups, [NCEP g2c's decoder](https://github.com/NOAA-EMC/NCEPLIBS-g2c/blob/develop/src/comunpack.c) and [ecCodes](https://github.com/ecmwf/eccodes/blob/develop/src/eccodes/accessor/DataG22OrderPacking.cc) return the reference unscaled. This matches [g2c's encoder](https://github.com/NOAA-EMC/NCEPLIBS-g2c/blob/develop/src/compack.c), which stores the unscaled constant as the reference. GDAL bundles g2c and agrees. [wgrib2](https://github.com/NOAA-EMC/wgrib2/blob/develop/src/unpk_complex.c) instead applies the decimal scale, so the decoders differ when both the reference and decimal scale are nonzero. Both real fixtures have decimal scale zero; only the synthetic cases distinguish these conventions.
