#[macro_use]
extern crate grib_data_derive;

use grib_data_derive::DisplayDescription;

#[derive(DisplayDescription)]
enum Shape {
    #[desc = "rectangle"]
    Rectangle = 0,
    #[desc = "triangle"]
    Triangle = 1,
    #[desc = "circle"]
    Circle = 2,
}

#[test]
fn display_description() {
    let r = Shape::Rectangle;
    let desc = r.to_string();
    assert_eq!(desc, "test");
}