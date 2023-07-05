use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use structopt::clap::arg_enum;
use structopt::StructOpt;
use walkdir::WalkDir;
use zip::write::{FileOptions, ZipWriter};
use zip::{CompressionMethod, DateTime};

arg_enum! {
    #[derive(Debug)]
pub enum Compression {
    None,
    Deflate,
}
}


impl Into<CompressionMethod> for Compression {
    fn into(self) -> CompressionMethod {
        match self {
            Compression::None => CompressionMethod::Stored,
            Compression::Deflate => CompressionMethod::Deflated,
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt()]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    pub output: PathBuf,

    #[structopt(
        short,
        long,
        possible_values = &Compression::variants(),
        case_insensitive = true,
        default_value = "Deflate"
    )]
    pub compression: Compression,

    #[structopt(short, long)]
    pub quiet: bool,

    #[structopt(parse(from_os_str), required(true))]
    pub paths: Vec<PathBuf>,
}

pub fn handle_path(path: PathBuf) -> Vec<PathBuf> {
    if path.is_file() {
        vec![path]
    } else {
        WalkDir::new(&path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.into_path())
            .collect()
    }
}

pub fn create_zip_file<W>(
    output_file: W,
    mut paths: Vec<(PathBuf, PathBuf)>,
    compression: CompressionMethod,
    quiet: bool,
) -> Result<(), std::io::Error>
where
    W: Write + Seek,
{
    paths.sort();
    let options = FileOptions::default()
        .last_modified_time(DateTime::default())
        .compression_method(compression);
    let mut zip_writer = ZipWriter::new(output_file);

    let mut buffer = Vec::new();

    for (name, path) in paths {
        if !quiet {
            println!("{}", path.display());
        }
        if path.is_dir() {
            if path.as_os_str().is_empty() {
                continue;
            }
            zip_writer.add_directory_from_path(name.as_path(), options)?;
        } else {
            zip_writer.start_file_from_path(name.as_path(), options)?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip_writer.write_all(&*buffer)?;
            buffer.clear();
        }
    }

    zip_writer.finish()?;
    Ok(())
}

pub fn zip_file(args: Opt) -> Result<(), std::io::Error> {
    let paths: Vec<(PathBuf, PathBuf)> = args
        .paths
        .into_iter()
        .flat_map(handle_path)
        .map(|p| (p.clone(), p))
        .collect();
    let output_file = File::create(args.output)?;
    create_zip_file(output_file, paths, args.compression.into(), args.quiet)?;
    Ok(())
}

