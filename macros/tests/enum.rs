use gribberish_macros::{DisplayDescription, FromAbbrevStr, FromValue, ToParameter};
use gribberish_types::Parameter;
use std::str::FromStr;

#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue, ToParameter, FromAbbrevStr)]
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

#[test]
fn shape_from_abbrev_str() {
    assert_eq!(Shape::from_str("rect"), Ok(Shape::Rectangle));
    assert_eq!(Shape::from_str("tri"), Ok(Shape::Triangle));
    assert_eq!(Shape::from_str("cir"), Ok(Shape::Circle));
    assert!(Shape::from_str("foo").is_err());
}
