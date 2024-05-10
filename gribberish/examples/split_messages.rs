use std::fs::File;
use std::io::{Read, Write};
use std::path::{PathBuf};
use clap::Parser;
use gribberish::message_metadata::scan_message_metadata;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output folder
    #[arg(short = 'o')]
    output_folder: Option<String>,

    /// Input file
    #[arg(value_name = "INPUT_FILE", index = 1)]
    input_file: String,
}

impl Args {
    pub fn output_folder(&self) -> String {
        if let Some(folder) = self.output_folder.as_ref() {
            folder.clone()
        } else {
            let input_path = PathBuf::from(&self.input_file);
            let grib_dir = input_path.parent().expect("Failed to get grib output folder");
            grib_dir.to_str().expect("Failed to get grib output folder path").into()
        }
    }
}

pub fn main() {
    let args = Args::parse();

    let grib_path = PathBuf::from(&args.input_file);
    let mut grib_file = File::open(&grib_path).expect("file not found");

    let grib_name = grib_path.file_stem().expect("Failed to get grib filename");
    let output_folder = args.output_folder();

    let mut raw_grib_data = Vec::new();
    grib_file
        .read_to_end(&mut raw_grib_data)
        .expect("failed to read raw grib2 data");

    let messages = scan_message_metadata(raw_grib_data.as_slice());
    messages.iter().for_each(|(key, (idx, offset, metadata))| {
        let mut message_path = PathBuf::from(&output_folder);
        let root_name = grib_name.to_str().expect("Failed to get filename string");
        let filename = format!("{root_name}-{key}-{idx}.grib2");
        message_path.push(filename);
        let mut message_file = File::create(message_path).expect("Failed to create message file");

        let data = &raw_grib_data[*offset..*offset+metadata.message_size];

        message_file.write_all(&data).expect("Failed to write message file");
    });
}
