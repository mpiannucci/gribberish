use gribberish_macros::{DisplayDescription, FromValue, ToParameter};
use gribberish_types::Parameter;

#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter)]
enum Shape {
    #[description = "rectangle"]
    #[abbrev = "rect"]
    #[unit = "sqft"]
    Rectangle = 0,
    #[description = "triangle"]
    #[abbrev = "tri"]
    #[unit = "sqft"]
    Triangle = 1,
    #[name = "round one"]
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
    assert_eq!(triangle.name(), "triangle");

    let circle: Shape = 2u8.into();
    assert_eq!(circle.unit(), "radius");
    assert_eq!(circle.name(), "round one");
}