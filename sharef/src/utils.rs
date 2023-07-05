mod deterministic_zip;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::Cursor;
use sha2::{Digest, Sha256};
use local_ip_address::local_ip;
use std::path::Path;
use deterministic_zip::{zip_file, Opt, Compression};
use zip_extract;

pub fn reconstruct_ip(small_ip: String) -> String {
    let ip = String::from("192.168.");
    let small_ip = small_ip;
    let mut ip = ip + &small_ip[0..2] + "." + &small_ip[2..4];

    ip = ip + ":7878";
    ip
}

pub fn calculate_file_hash(file_path: &str) -> io::Result<String> {

    let mut file = File::open(file_path)?;


    let mut hasher = Sha256::new();


    let mut buffer = [0u8; 4096];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }


    let hash_result = hasher.finalize();


    let hash_string = format!("{:x}", hash_result);

    Ok(hash_string)
}

pub fn get_first_hostname_ip() -> String {
    let my_local_ip = local_ip().unwrap();
    let ip_address = my_local_ip.to_string();
    ip_address
}

pub fn reduce_ip(ip_address: &str) -> String {

    let mut ip_address = ip_address.replace(".", "");

    ip_address = ip_address.replace("192168", "");
    ip_address
}

pub fn create_zip(path: &str) -> String{
    let path = Path::new(path);

    let output = Path::new(&std::env::current_dir().unwrap())
        .join(path.file_name().unwrap())
        .with_extension("zip");

    let filename = path.file_name().unwrap().to_str().unwrap();

    println!("Creating zip file: {}", output.to_str().unwrap());
    let opt = Opt {
        output,
        compression: Compression::Deflate,
        quiet: true,
        paths: vec![path.to_path_buf()],
    };

    zip_file(opt).unwrap();

    return filename.to_string() + ".zip";
}

pub fn extract_zip(filename: &str){
    let file = filename.replace(".zip", "");
    let file = Path::new(&file);

    let target = Path::new(&std::env::current_dir().unwrap())
        .join(file.file_name().unwrap());

    let archive: Vec<u8> = std::fs::read(filename).unwrap();

    println!("Extracting zip file: {}", filename);

    zip_extract::extract(Cursor::new(archive), &target, true).unwrap();
}

pub fn clean_up(filename: &str) {
    let filename = if !filename.ends_with(".zip") {
        filename.to_string() + ".zip"
    } else {
        filename.to_string()
    };

    let path = Path::new(&filename);

    if path.exists() {
        println!("Deleting file: {}", filename);
        std::fs::remove_file(filename).unwrap();
    }
}


