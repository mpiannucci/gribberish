extern crate chrono;
extern crate gribberish;
extern crate futures;
extern crate tokio;
extern crate reqwest;
extern crate bytes;
extern crate csv;

use std::error::Error;
use std::fmt;
use std::clone::Clone;
use std::ops::Range;
use std::time::{Duration, Instant};
use futures::{stream, StreamExt};
use chrono::prelude::*;
use reqwest::{Url};
use bytes::Bytes;

#[derive(Clone, Debug)]
enum NOAAModelType {
    MultiGridWave,
    NWW3MultiWave,
}

impl NOAAModelType {
    pub fn filter_name(&self) -> &'static str {
        match self {
            NOAAModelType::MultiGridWave => "wave_multi",
            NOAAModelType::NWW3MultiWave => "wave",
        }
    }
}

impl fmt::Display for NOAAModelType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            NOAAModelType::MultiGridWave => "multi_1",
            NOAAModelType::NWW3MultiWave => "multi_1",
        };
        write!(f, "{}", name)
    }
}

#[derive(Clone, Debug)]
struct NOAAModelUrlBuilder<'a> {
    model_type: NOAAModelType,
    model_region_name: &'a str,
    date: DateTime<Utc>,
    index: Option<usize>,
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
            index: None,
            subregion: None,
            variables: vec![],
        }
    }

    pub fn at_index(&mut self, index: usize) -> &mut Self {
        self.index = Some(index);
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
        format!("https://nomads.ncep.noaa.gov/cgi-bin/filter_{}.pl?file={}.{}.t{:02}z{}.grib2&all_lev=on{}{}&dir=%2F{}.{}", 
            self.model_type.filter_name(), 
            self.model_type, 
            self.model_region_name, 
            self.date.hour(),
            self.build_index(),
            self.build_vars(),
            self.build_subregion(),
            self.model_type, 
            self.date.format("%Y%m%d"),
        )
    }

    pub fn build_at_indexes(&self, indexes: Range<usize>) -> Vec<String> {
        let mut builder = self.clone();
        indexes.step_by(1).filter_map(|i| {
            if i > 120 && (i - 120) % 3 != 0 {
                return None;
            }

            builder
                .at_index(i);
            Some(builder.build())
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

    fn build_index(&self) -> String {
        if let Some(index) = self.index {
            format!(".f{:03}", index)
        } else {
            String::from("")
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

pub fn mean(data: &Vec<f64>) -> f64 {
    let filtered_data: Vec<_> = data
        .iter()
        .filter(|v| !v.is_nan())
        .collect();

    let count = filtered_data.len() as f64;
    filtered_data.into_iter().sum::<f64>() / count
}

// URL SCHEMES
// https://nomads.ncep.noaa.gov/cgi-bin/filter_gfs_0p25_1hr.pl?file=gfs.t12z.pgrb2.0p25.f000&leftlon=0&rightlon=360&toplat=90&bottomlat=-90&dir=%2Fgfs.20201005%2F12
// https://nomads.ncep.noaa.gov/cgi-bin/filter_wave_multi.pl?file=multi_1.at_10m.t06z.f040.grib2&leftlon=0&rightlon=360&toplat=90&bottomlat=-90&dir=%2Fmulti_1.20201005
// https://nomads.ncep.noaa.gov/cgi-bin/filter_wave.pl?file=multi_1.nww3.t06z.grib2&all_lev=on&var_SWDIR=on&var_SWELL=on&leftlon=0&rightlon=360&toplat=90&bottomlat=-90&dir=%2Fmulti_1.20201005

// RI Coast 41.4, -71.45l
// BI Buoy 40.969, 71.127
// https://nomads.ncep.noaa.gov/cgi-bin/filter_gfs_0p50.pl?file=gfs.t06z.pgrb2full.0p50.f168&lev_10_m_above_ground=on&var_GUST=on&var_PRES=on&var_TMP=on&var_UGRD=on&var_VGRD=on&subregion=&leftlon=-72.0&rightlon=-71.0&toplat=42.0&bottomlat=41.0&dir=%2Fgfs.20200909%2F06
// https://nomads.ncep.noaa.gov/cgi-bin/filter_wave_multi.pl?file=multi_1.at_10m.t06z.f057.grib2&all_lev=on&all_var=on&subregion=&leftlon=-72.0&rightlon=-71.0&toplat=42.0&bottomlat=41.0&dir=%2Fmulti_1.20200909
// https://nomads.ncep.noaa.gov/cgi-bin/filter_wave.pl?file=multi_1.nww3.t12z.grib2&subregion=&leftlon=265&rightlon=275&toplat=42&bottomlat=41&dir=%2Fmulti_1.20201209

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let start = Instant::now();

    let now = Utc::now().with_hour(6).unwrap();
    let urls = NOAAModelUrlBuilder::new(NOAAModelType::MultiGridWave, "at_10m", now)
        .with_subregion(41.0, 42.0, -72.0, -71.0)
        .build_at_indexes(0..180);

    println!("Create Urls: {:?}", start.elapsed());

    // Download the data from NOAA's grib endpoint
    let results: Vec<Option<Bytes>> = stream::iter(urls.into_iter().map(|url|
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
    })).buffered(8).collect().await;

    println!("Download Data: {:?}", start.elapsed());
    
    // Parse out the data into data and metadata
    let all_grib_data: Vec<_> = results
        .into_iter()
        .filter_map(|b| {
            match b {
                Some(b) => {
                    let data: Vec<_> = gribberish::message::Message::parse_all(b.clone().as_ref())
                    .iter()
                    .filter(|m| m.metadata().is_ok())    
                    .map(|m| (m.metadata().unwrap(), m.data(), m.data_locations()))
                    .collect();
                    Some(data)
                },
                None => None,
            }
        }).collect();

    println!("Parse Model Data: {:?}", start.elapsed());
    
    let mut wtr = csv::Writer::from_path("./examples/grib_filter/output/ri_wave_data.csv")?;

    // Collect the variables and write out the result as the header
    let mut vars: Vec<_> = all_grib_data[0]
        .iter()
        .map(|m| format!("{} ({})", (m.0).variable_abbreviation.clone(), (m.0).units ))
        .collect();
    if vars.len() == 0 {
        return Err(Box::from("No variables read"));
    }
    vars.insert(0, String::from("TIME"));
    wtr.write_record(vars)?;

    // Then collect the mean of every value 
    all_grib_data.iter().for_each(|dt| {
        let mut point_data: Vec<_> = dt
            .iter()
            .map(|d| {
                let value = match &d.1 {
                    Ok(vals) => mean(vals),
                    Err(_) => std::f64::NAN,
                };
                format!("{:.2}", value)
            }).collect();
        if point_data.len() > 0 {
            point_data.insert(0, dt[0].0.forecast_date.to_rfc3339());
        }

        let _ = wtr.write_record(point_data);
    });

    wtr.flush()?;

    println!("Output Model Data: {:?}", start.elapsed());

    Ok(())
}
