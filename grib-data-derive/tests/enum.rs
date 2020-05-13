#[macro_use]
extern crate grib_data_derive;

use grib_data_derive::DisplayDescription;

#[derive(DisplayDescription)]
enum Shape {
    #[desc = "rectangle"]
    Rectangle = 0,
    #[desc = "triangle"]
    Triangle = 1,
    Circle = 2,
}

#[test]
fn display_description() {
    let rect = Shape::Rectangle;
    let rect_desc = rect.to_string();
    assert_eq!(rect_desc, "rectangle");

    let circle = Shape::Circle;
    let circle_desc = circle.to_string();
    assert_eq!(circle_desc, "Circle");
}