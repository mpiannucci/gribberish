pub struct Parameter {
    pub name: String,
    pub unit: String,
    pub abbrev: String,
}


#[derive(Debug, PartialEq, Eq)]
pub struct ParseAbbrevError(pub String);

