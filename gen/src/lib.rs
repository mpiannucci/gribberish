use std::num::ParseIntError;

use quote::quote;

/// Parameter record table: https://github.com/weathersource/wgrib2/blob/master/wgrib2/gribtables/ndfd/NDFD_gribtable.dat
///  int disc;   Section 0 Discipline
///  int mtab_set;   Section 1 Master Tables Version Number
///  int mtab_low;   Section 1 Master Tables Version Number
///  int mtab_high;   Section 1 Master Tables Version Number
///  int cntr;   Section 1 originating centre, used for local tables
///  int ltab;   Section 1 Local Tables Version Number
///  int pcat;   Section 4 Template 4.0 Parameter category
///  int pnum;   Section 4 Template 4.0 Parameter number
///  const char *name;
///  const char *desc;
///  const char *unit;
pub struct ParameterRecord {
    pub discipline: u8,
    pub parameter_category: u8,
    pub parameter_number: u8,
    pub name: String,
    pub description: String,
    pub unit: String,
}

impl ParameterRecord {
    pub fn from_dat(dat: &str) -> Result<ParameterRecord, ParseIntError>  {
        let dat = dat.trim().replace("{", "").replace("}", "");
        let components = dat.split(",").collect::<Vec<&str>>();
        let discipline = components[0].trim().parse::<u8>()?;
        // let master_table_version_set = components[1].trim().parse::<u8>().unwrap();
        // let master_table_version_low = components[2].trim().parse::<u8>().unwrap();
        // let master_table_version_high = components[3].trim().parse::<u8>().unwrap();
        // let originating_centre: u8 = components[4].trim().parse::<u8>().unwrap();
        // let local_table_version = components[5].trim().parse::<u8>().unwrap();
        let parameter_category = components[6].trim().parse::<u8>()?;
        let parameter_number = components[7].trim().parse::<u8>()?;
        let name = components[8].trim().replace("\"", "").to_string();
        let description = components[9].trim().replace("\"", "").to_string();
        let unit = components[10].trim().replace("\"", "").to_string();

        // let master_table_version = format!("{master_table_version_set}.{master_table_version_low}");
        Ok(Self {
            discipline,
            parameter_category,
            parameter_number,
            name,
            description,
            unit,
        })
    }
}

pub struct ParameterTable {
    records: Vec<ParameterRecord>,
}

impl ParameterTable {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: ParameterRecord) {
        self.records.push(record);
    }

    pub fn add_from_dat_file(&mut self, fate_filename: &str) {
        let dat = std::fs::read_to_string(fate_filename).unwrap();
        let lines = dat.split("\n").collect::<Vec<&str>>();
        for line in lines {
            if let Ok(record) = ParameterRecord::from_dat(line) {
                self.records.push(record);
            }
        }
    }

    pub fn generate(&self) -> String {
        let mut builder = phf_codegen::Map::new();
        for record in self.records.iter() {
            let discipline = record.discipline;
            let category = record.parameter_category;
            let number = record.parameter_number;
            let key = format!("{discipline}-{category}-{number}");

            let name = record.name.as_str();
            let description = record.description.as_str();
            let unit = record.unit.as_str();
            let def = quote! {
                Parameter {
                    name: #name,
                    description: #description,
                    unit: #unit,
                }
            };
            builder.entry(key, &format!("{def}"));
        }

        format!("pub static PARAMETER_TABLE: phf::Map<&'static str, Parameter> = {}", builder.build())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_parameter_record() {
        let dat = r#"{0,1,0,255,8,1,1,192, "WX", "Weather information", "WxInfo"}, "#;
        let record = ParameterRecord::from_dat(dat).expect("Should Parse");

        assert_eq!(record.discipline, 0);
        assert_eq!(record.parameter_category, 1);
        assert_eq!(record.parameter_number, 192);
        assert_eq!(record.name, "WX");
        assert_eq!(record.description, "Weather information");
        assert_eq!(record.unit, "WxInfo");
    }

    #[test]
    fn test_explicit_parse_to_table() {
        let dat = r#"{209,10,0,255,161,1,6,2, "RadarOnlyQPE01H", "Radar precipitation accumulation 1-hour", "mm"}, "#;
        let record = ParameterRecord::from_dat(dat).expect("Should parse");
        let mut table = ParameterTable::new();
        table.add_record(record);
        let table = table.generate();
        assert!(table.contains("RadarOnlyQPE01H"));
    }

    #[test]
    fn test_parse_file_to_table() {
        let mut table = ParameterTable::new();
        table.add_from_dat_file("./../tables/MRMS_gribtable.dat");
        let table = table.generate();
        assert!(table.contains("RadarOnlyQPE01H"));
    }
}