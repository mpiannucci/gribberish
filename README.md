# gribberish

Read [GRIB 2](https://en.wikipedia.org/wiki/GRIB) files with Rust. No `C` required.

## Motivation

This crate is inspired by [grippy](https://github.com/mpiannucci/grippy), a pure python grib 2 parser. Traditionally GRIB files are used by Weather modelling programs to efficiently store and transfer data. However, the only widely used parsers for these systems are written with C and C++ and require a long stack of dependencies that can be hard to wrangle outside of Linux systems. 

While this is fine for most people, it makes it more difficult to use these parsers with newer languages or cloud systems like Google App Engine. For python, it used to be required on Google App Engine that all python dependencies could not use native code. For this reason I wrote a pure python GRIB parser to create my own surf forecasts from.

The limitation is that while python is easy, it is also slow. I want to speed things up and see how fast I can make a rust parser and learn more advanced topics in the language along the way. This will hopefully also make this forecast data easier to retreive for newer, non system admin users. 

## References and Spec

https://www.wmo.int/pages/prog/www/WMOCodes/Guides/GRIB/GRIB2_062006.pdf

https://www.nco.ncep.noaa.gov/pmb/docs/grib2/grib2_doc/

https://www.wmo.int/pages/prog/www/DPS/FM92-GRIB2-11-2003.pdf