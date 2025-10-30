use gribberish_macros::{DisplayDescription, FromValue};

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum OriginalFieldValue {
    FloatingPoint = 0,
    Integer = 1,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum MatrixCoordinateValueFunctions {
    ExplicitCoordinateValueSet = 0,
    LinearCoordinates = 1,
    GeometricCoordinates = 11,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum MatrixCoordinateParameters {
    DirectionDegreesTrue = 1,
    Frequency = 2,
    RadialNumber = 3,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum GroupSplittingMethod {
    RowByRow = 0,
    GeneralGroup = 1,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum MissingValueManagement {
    #[description = "no explicit missing values included with the data values"]
    NoMissingValues = 0,
    #[description = "primary missing values included within the data value"]
    IncludesMissingPrimary = 1,
    #[description = "primary and secondary missing values included within the data values"]
    IncludesMissingPrimarySecondary = 2,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum SpatialDifferencingOrder {
    #[description = "first order spatial differencing"]
    First = 1,
    #[description = "second order spatial differencing"]
    Second = 2,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum FloatingPointPrecision {
    #[description = "IEEE 32 bit"]
    IEEE32Bit = 1,
    #[description = "IEEE 64 bit"]
    IEEE64Bit = 2,
    #[description = "IEEE 128 bit"]
    IEEE128Bit = 3,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum CompressionType {
    Lossless = 0,
    Lossy = 1,
}