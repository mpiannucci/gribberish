#[macro_use]
extern crate grib_data_derive;

use grib_data_derive::{DisplayDescription, FromValue, Parameter};

#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, Parameter)]
enum Shape {
    #[description = "rectangle"]
    #[abbrev = "rect"]
    #[unit = "sqft"]
    Rectangle = 0,
    #[description = "triangle"]
    #[abbrev = "tri"]
    #[unit = "sqft"]
    Triangle = 1,
    #[abbrev = "cir"]
    #[unit = "radius"]
    Circle = 2,
}

#[test]
fn shape_display_description() {
    let rect = Shape::Rectangle;
    let rect_desc = rect.to_string();
    assert_eq!(rect_desc, "rectangle");

    let circle = Shape::Circle;
    let circle_desc = circle.to_string();
    assert_eq!(circle_desc, "circle");
}

#[test]
fn shape_from_value() {
    let rectangle: Shape = 0.into();
    assert_eq!(rectangle, Shape::Rectangle);

    let triangle: Shape = 1u8.into();
    assert_eq!(triangle, Shape::Triangle);

    let circle: Shape = 2u8.into();
    assert_eq!(circle, Shape::Circle);
}

#[test]
fn shape_parameter_attributes() {
    let rectangle: Shape = 0.into();
    assert_eq!(rectangle.abbrev(), "rect");

    let triangle: Shape = 1u8.into();
    assert_eq!(triangle.abbrev(), "tri");

    let circle: Shape = 2u8.into();
    assert_eq!(circle.unit(), "radius");
}