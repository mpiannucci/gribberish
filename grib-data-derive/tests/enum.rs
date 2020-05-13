#[macro_use]
extern crate grib_data_derive;

use grib_data_derive::{DisplayDescription, FromValue};

#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
enum Shape {
    #[description = "rectangle"]
    Rectangle = 0,
    #[description = "triangle"]
    Triangle = 1,
    Circle = 2,
}

#[test]
fn shape_display_description() {
    let rect = Shape::Rectangle;
    let rect_desc = rect.to_string();
    assert_eq!(rect_desc, "rectangle");

    let circle = Shape::Circle;
    let circle_desc = circle.to_string();
    assert_eq!(circle_desc, "Circle");
}

#[test]
fn shape_from_value() {
    let triangle: Shape = 1u8.into();
    assert_eq!(triangle, Shape::Triangle);
}