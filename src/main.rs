#[macro_use]
extern crate log;

use anyhow::Result;
use argh::FromArgs;
use blake2::{Blake2s, Digest};
use hex::encode;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(FromArgs)]
/// A simple calculation tool
struct MyCli {
    #[argh(subcommand)]
    subcommand: SubCommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommands {
    Scan(ScanOptions),
    Blobs(BlobsOptions),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Scan file system, generate checksums
#[argh(subcommand, name = "scan")]
pub struct ScanOptions {
    #[argh(option)]
    /// path to scan
    path: PathBuf,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Query OpenStack Swift for blobs
#[argh(subcommand, name = "blobs")]
pub struct BlobsOptions {
    #[argh(option)]
    /// swift API url
    url: String,
    #[argh(option)]
    /// swift API password
    password: String,
}

pub struct FileInfo {
    pub path: PathBuf,
    pub size: usize,
    pub checksum: String,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli: MyCli = argh::from_env();
    match cli.subcommand {
        SubCommands::Scan(options) => {
            let res = scan(&options.path)?;
            info!("Checksum for {} is {}", options.path.to_string_lossy(), res)
        }
        SubCommands::Blobs(options) => info!("URL {}, password {}", options.url, options.password),
    };

    Ok(())
}

fn scan(path: &Path) -> Result<String> {
    info!("scan path {}", path.to_string_lossy());

    // create a Blake2s hasher
    // better than SHA-1, reasonably small (256) hash size compared to Blake2d
    let mut hasher = Blake2s::new();

    const BUFFER_SIZE: usize = 1024 * 128;
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);

    loop {
        let length = {
            let buffer = reader.fill_buf()?;
            hasher.update(buffer);
            buffer.len()
        };
        if length == 0 {
            break;
        }
        reader.consume(length);
    }

    Ok(encode(hasher.finalize()))
}
