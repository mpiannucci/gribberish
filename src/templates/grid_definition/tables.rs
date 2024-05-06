use gribberish_macros::{DisplayDescription, FromValue};

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum EarthShape {
    #[description = "Earth assumed spherical with radius = 6,367,470.0 m"]
    Spherical = 0,
    #[description = "Earth assumed spherical with radius specified (in m) by data producer"]
    SpecifiedRadiusSpherical = 1,
    #[description = "Earth assumed oblate spheroid with size as determined by IAU in 1965 (major axis = 6,378,160.0 m, minor axis = 6,356,775.0 m, f = 1/297.0) "]
    OblateIAU = 2,
    #[description = "Earth assumed oblate spheroid with major and minor axes specified (in km) by data producer"]
    OblateKM = 3,
    #[description = "Earth assumed oblate spheroid as defined in IAG-GRS80 model (major axis = 6,378,137.0 m, minor axis = 6,356,752.314 m, f = 1/298.257222101) "]
    OblateIAGGRS80 = 4,
    #[description = "Earth assumed represented by WGS84 (as used by ICAO since 1998) "]
    WGS84 = 5,
    #[description = "Earth assumed spherical with radius of 6,371,229.0 m"]
    Spherical2 = 6,
    #[description = "Earth assumed oblate spheroid with major and minor axes specified (in m) by data producer "]
    OblateM = 7,
    #[description = "Earth model assumed spherical with radius 6371200 m, but the horizontal datum of the resulting latitude/longitude field is the WGS84 reference frame"]
    OblateWGS84 = 8,
    Missing = 255,
}

#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum ProjectionCenter {
    #[description = "North Pole is on the projection plane"]
    NorthPole,
    #[description = "South Pole is on the projection plane"]
    SouthPole,
    #[description = "Only one projection center is used"]
    OneCenter,
    #[description = "Projection is bi-polar and symmetric"]
    BiPolar,
}

pub type ProjectionCenterFlags = [ProjectionCenter; 2];

impl ProjectionCenter {
    pub fn read_flags(data: u8) -> ProjectionCenterFlags {
        let pole = match data & 128 == 0 {
            true => ProjectionCenter::NorthPole,
            false => ProjectionCenter::SouthPole,
        };
        let center = match data & 64 == 0 {
            true => ProjectionCenter::OneCenter,
            false => ProjectionCenter::BiPolar,
        };

        [pole, center]
    }
}

#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum ScanningMode {
    #[description = "Points in the first row or column scan in the +i (+x) direction"]
    PlusI,
    #[description = "Points in the first row or column scan in the -i (-x) direction"]
    MinusI,
    #[description = "Points in the first row or column scan in the -j (-y) direction"]
    MinusJ,
    #[description = "Points in the first row or column scan in the +j (+y) direction"]
    PlusJ,
    #[description = "Adjacent points in the i (x) direction are consecutive"]
    ConsecutiveI,
    #[description = "Adjacent points in the j (y) direction are consecutive"]
    ConsecutiveJ,
    #[description = "All rows scan in the same direction"]
    SameDirection,
    #[description = "Adjacent rows scan in the opposite direction"]
    AdjecentOppositeDirection,
    #[description = "Points within odd rows are not offset in i(x) direction"]
    OddNotOffset,
    #[description = "Points within odd rows are offset by Di/2 in i(x) direction"]
    OddOffset,
    #[description = "Points within even rows are not offset in i(x) direction"]
    EvenNotOffset,
    #[description = "Points within even rows are offset by Di/2 in i(x) direction"]
    EvenOffset,
    #[description = "Points are not offset in j(y) direction"]
    NotOffsetJ,
    #[description = "Points are offset by Dj/2 in j(y) direction"]
    OffsetJ,
    #[description = "Rows have Ni grid points and columns have Nj grid points"]
    Normal,
    #[description = "Rows have Ni grid points if points are not offset in i direction, 
    Rows have Ni-1 grid points if points are offset by Di/2 in i direction, 
    Columns have Nj grid points if points are not offset in j direction, 
    Columns have Nj-1 grid points if points are offset by Dj/2 in j direction"]
    Staggerred,
}

pub type ScanningModeFlags = [ScanningMode; 8];

impl ScanningMode {
    pub fn read_flags(data: u8) -> ScanningModeFlags {
        let first = match data & 128 == 0 {
            true => ScanningMode::PlusI,
            false => ScanningMode::MinusI,
        };
        let second = match data & 64 == 0 {
            true => ScanningMode::MinusJ,
            false => ScanningMode::PlusJ,
        };
        let third = match data & 32 == 0 {
            true => ScanningMode::ConsecutiveI,
            false => ScanningMode::ConsecutiveJ,
        };
        let fourth = match data & 16 == 0 {
            true => ScanningMode::SameDirection,
            false => ScanningMode::AdjecentOppositeDirection,
        };
        let fifth = match data & 8 == 0 {
            true => ScanningMode::OddNotOffset,
            false => ScanningMode::OddOffset,
        };
        let sixth = match data & 4 == 0 {
            true => ScanningMode::EvenNotOffset,
            false => ScanningMode::EvenOffset,
        };
        let seventh = match data & 2 == 0 {
            true => ScanningMode::NotOffsetJ,
            false => ScanningMode::OffsetJ,
        };
        let eigth = match data & 1 == 0 {
            true => ScanningMode::Normal,
            false => ScanningMode::Staggerred,
        };

        [first, second, third, fourth, fifth, sixth, seventh, eigth]
    }
}

#[cfg(test)]
mod tests {
    use crate::templates::grid_definition::tables::ScanningMode;

    use super::ProjectionCenter;

    #[test]
    fn parse_projection_center() {
        let first = 0b00000000_u8;
        let flags = ProjectionCenter::read_flags(first);
        assert_eq!(flags[0], ProjectionCenter::NorthPole);
        assert_eq!(flags[1], ProjectionCenter::OneCenter);

        let second = 0b10000001_u8;
        let flags = ProjectionCenter::read_flags(second);
        assert_eq!(flags[0], ProjectionCenter::SouthPole);
        assert_eq!(flags[1], ProjectionCenter::OneCenter);

        let third = 0b11000000_u8;
        let flags = ProjectionCenter::read_flags(third);
        assert_eq!(flags[0], ProjectionCenter::SouthPole);
        assert_eq!(flags[1], ProjectionCenter::BiPolar);

        let fourth = 0b01000000_u8;
        let flags = ProjectionCenter::read_flags(fourth);
        assert_eq!(flags[0], ProjectionCenter::NorthPole);
        assert_eq!(flags[1], ProjectionCenter::BiPolar);
    }

    #[test]
    fn parse_scanning_mode() {
        let first = 0b00000000_u8;
        let flags = ScanningMode::read_flags(first);
        assert_eq!(flags[0], ScanningMode::PlusI);
        assert_eq!(flags[1], ScanningMode::MinusJ);

        let second = 0b10000000_u8;
        let flags = ScanningMode::read_flags(second);
        println!("{:?}", flags);
        assert_eq!(flags[0], ScanningMode::MinusI);
        assert_eq!(flags[1], ScanningMode::MinusJ);
    }
}
