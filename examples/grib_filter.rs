extern crate chrono;
extern crate grib;
extern crate futures;
extern crate tokio;
extern crate reqwest;
extern crate bytes;

use std::fmt;
use std::clone::Clone;
use std::ops::Range;
use futures::{stream, StreamExt};
use futures::stream::Collect;
use chrono::prelude::*;
use reqwest::{Url};
use bytes::Bytes;

#[derive(Clone, Debug)]
enum NOAAModelType {
    MultiGridWave,
}

impl NOAAModelType {
    pub fn filter_name(&self) -> &'static str {
        match self {
            NOAAModelType::MultiGridWave => "wave_multi",
        }
    }
}

impl fmt::Display for NOAAModelType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            NOAAModelType::MultiGridWave => "multi_1",
        };
        write!(f, "{}", name)
    }
}

#[derive(Clone, Debug)]
struct NOAAModelUrlBuilder<'a> {
    model_type: NOAAModelType,
    model_region_name: &'a str,
    date: DateTime<Utc>,
    index: usize,
    subregion: Option<((f64, f64), (f64, f64))>,
    variables: Vec<String>,
}

impl<'a> NOAAModelUrlBuilder<'a> {
    pub fn new(
        model_type: NOAAModelType,
        model_region_name: &'a str,
        date: DateTime<Utc>,
    ) -> Self {
        NOAAModelUrlBuilder {
            model_type,
            model_region_name,
            date,
            index: 0,
            subregion: None,
            variables: vec![],
        }
    }

    pub fn at_index(&mut self, index: usize) -> &mut Self {
        self.index = index;
        self
    }

    pub fn with_subregion(
        &mut self,
        min_lat: f64,
        max_lat: f64,
        min_lng: f64,
        max_lng: f64,
    ) -> &mut Self {
        self.subregion = Some(((min_lat, min_lng), (max_lat, max_lng)));
        self
    }

    pub fn with_var(&mut self, var: String) -> &mut Self {
        if !self.variables.contains(&var) {
            self.variables.push(var);
        }
        self
    }

    pub fn with_vars(&mut self, vars: Vec<String>) -> &mut Self {
        for var in vars {
            if !self.variables.contains(&var) {
                self.variables.push(var);
            }
        }
        self
    }

    pub fn build(&self) -> String {
        format!("https://nomads.ncep.noaa.gov/cgi-bin/filter_{}.pl?file={}.{}.t{:02}z.f{:03}.grib2{}{}&dir=%2F{}.{}", 
            self.model_type.filter_name(), 
            self.model_type, 
            self.model_region_name, 
            self.date.hour(),
            self.index,
            self.build_vars(),
            self.build_subregion(),
            self.model_type, 
            self.date.format("%Y%m%d"),
        )
    }

    pub fn build_at_indexes(&self, indexes: Range<usize>) -> Vec<String> {
        let mut builder = self.clone();
        indexes.map(|i| {
            builder
                .at_index(i);
            builder.build()
        }).collect()
    }

    fn build_subregion(&self) -> String {
        if let Some(region) = self.subregion {
            format!(
                "&subregion=&leftlon={}&rightlon={}&toplat={}&bottomlat={}",
                (region.0).1,
                (region.1).1,
                (region.1).0,
                (region.0).0
            )
        } else {
            String::new()
        }
    }

    fn build_vars(&self) -> String {
        if self.variables.len() > 0 {
        self.variables
            .iter()
            .map(|v| format!("&var_{}=on", *v))
            .collect::<Vec<String>>()
            .join("")
        } else {
            String::from("&all_var=on")
        }
    }
}

// RI Coast 41.4, -71.45
// BI Buoy 40.969, 71.127
#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let now = Utc::now().with_hour(0).unwrap();
    let urls = NOAAModelUrlBuilder::new(NOAAModelType::MultiGridWave, "at_10m", now)
        .with_subregion(41.4, 41.6, -71.6, -71.4)
        .build_at_indexes(0..181);

    let worker = stream::iter(urls.into_iter().map(|url|
        async move {
            let rurl = Url::parse(url.as_str()).unwrap();
            match reqwest::get(rurl).await {
                Ok(resp) => {
                    match resp.bytes().await {
                        Ok(b) => Some(b),
                        Err(_) => None,
                    }
                },
                Err(_) => None,
            }
    }));

    // worker.collect();
    
    // buffered(8).collect::<Vec<Option<Bytes>>>();

    // let data = worker.await;

    Ok(())
}
