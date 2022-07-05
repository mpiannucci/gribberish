use lazy_static::lazy_static;

use std::collections::HashMap; 
use std::sync::RwLock;

lazy_static! {
    static ref PARAMETER_TABLE: RwLock<HashMap<String, Parameter>> = {
        let table = HashMap::new();

        // TODO: Intialize with defualt WMO code values 

        RwLock::new(table)
    };
}

pub fn register_parameters(parameters: &HashMap<String, Parameter>) {
    if let Ok(mut parameter_table) = PARAMETER_TABLE.try_write() {
        for (key, parameter) in parameters {
            parameter_table.insert(key.to_string(), parameter.clone());
        }  
    } 
}

pub fn lookup_parameter(discipline: u8, category: u8, parameter: u8) -> Option<Parameter> {
    if let Ok(parameter_table) = PARAMETER_TABLE.try_read() {
        match parameter_table.get(&to_lookup_id(discipline, category, parameter)) {
            Some(p) => Some(p.clone()), 
            None => None,
        }
    } else {
        None
    }
}

pub fn to_lookup_id(discipline: u8, category: u8, parameter: u8) -> String {
    format!("{discipline}-{category}-{parameter}")
}

#[derive(Clone, Debug)]
pub struct Parameter {
    discipline: u8, 
    category: u8, 
    parameter: u8, 
    name: String, 
    description: String, 
    units: String, 
}

impl Parameter {
    pub fn id(&self) -> String {
        to_lookup_id(self.discipline, self.category, self.parameter)
    }
}
