use std::error;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Write};
use csv::ReaderBuilder;
use libflate::gzip::Decoder;
use serde::de::DeserializeOwned;
use serde_derive::Deserialize;
use semver::Version;
use tar::Archive;
use rayon::prelude::*;

const MAX_CRATE_SIZE: usize = 20 * 1000;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Deserialize, Debug)]
struct Crate {
    #[serde(rename = "id")]
    crate_id: u64,
    name: String,
    downloads: u64,
    description: Option<String>,
    repository: Option<String>,
    #[serde(skip_deserializing, default = "default_version")]
    version: Version,
}

#[derive(Deserialize, Debug)]
struct CrateVersion {
    crate_id: u64,
    num: Version,
}

fn default_version() -> Version {
    Version::parse("0.0.0").unwrap()
}

fn read_csv<D: DeserializeOwned>(file: impl Read) -> Result<Vec<D>> {
    let mut records: Vec<D> = vec![];
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);
    for record in reader.deserialize() {
        records.push(record?);
    }
    Ok(records)
}

/**
The purpose of executor:
from 052923 cratesio dbdump
get top 20k downloaded crates
write into list as .txt file
**/
fn executor() -> Result<()> {
    let mut crates: Vec<Crate> = Vec::new();
    let mut versions: Vec<CrateVersion> = Vec::new();

    let csv_path = "../db-dump.tar.gz";
    let mut archive = Archive::new(
        Decoder::new(
            BufReader::new(
                File::open(csv_path)?
            )
        )?
    );
    let entries = archive.entries()?.filter(|entry| {
        // Only filter the file we needed.
        entry
            .as_ref()
            .unwrap()
            .path()
            .unwrap()
            .file_name()
            .and_then(|f| f.to_str())
            .map(|f| ["crates.csv", "versions.csv"].contains(&f))
            .unwrap()
    });
    for file in entries {
        let file = file?;
        println!("{:?}", file.path()?);

        if let Some(filename) = file.path()?.file_name().and_then(|f| f.to_str()) {
            match filename {
                "crates.csv" => {
                    crates = read_csv(file)?;
                }
                "versions.csv" => {
                    versions = read_csv(file)?;
                }
                _ => {}
            }
        }
    }
    crates.par_sort_unstable_by(|a, b| b.downloads.cmp(&a.downloads));
    crates = crates.into_iter().take(MAX_CRATE_SIZE).collect();
    let file_name = "crates_list.txt";
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true)
        .open(file_name)?;
    writeln!(file, "crates:")?;
    for c in crates {
        writeln!(file, "{}", c.name);
    }
    Ok(())
}

fn main() {
    let _ = executor();
}