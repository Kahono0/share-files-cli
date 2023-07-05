mod deterministic_zip;

use local_ip_address::local_ip;
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::File;
use std::io::{Read, Cursor};
use std::io::{self, Write};
use std::io::{prelude::*, BufReader};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::Path;
use deterministic_zip::{Zip, Opt, Compression};
use zip_extract;



fn reconstruct_ip(small_ip: String) -> String {
    //example 3052
    //convert to 192.168.30.52 192.168 is the network address
    let ip = String::from("192.168.");
    let small_ip = small_ip;
    let mut ip = ip + &small_ip[0..2] + "." + &small_ip[2..4];
    //add port 7878
    ip = ip + ":7878";
    ip
}
fn handle_client(mut stream: TcpStream, filename: String) {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
                                  //
                                  //create file
    let mut file = File::create(filename).expect("create failed");
    println!("File created");
    while match stream.read(&mut data) {
        Ok(size) if size > 0 => {
            // Write the data to the file
            file.write_all(&data[0..size]).expect("write failed");
            true
        }
        Ok(_) => {
            println!("File received");
            println!("Checking checksum");
            //check checksum
            false
        }
        Err(_) => {
            println!(
                "An error occurred, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn calculate_file_hash(file_path: &str) -> io::Result<String> {
    // Open the file
    let mut file = File::open(file_path)?;

    // Create a SHA-256 hasher
    let mut hasher = Sha256::new();

    // Read the file in small chunks and update the hasher
    let mut buffer = [0u8; 4096];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    // Finalize the hash computation and obtain the hash value
    let hash_result = hasher.finalize();

    // Convert the hash value to a hexadecimal string representation
    let hash_string = format!("{:x}", hash_result);

    Ok(hash_string)
}

fn receive(small_ip: String) -> Result<String, String> {
    let server_address = reconstruct_ip(small_ip.to_string());

    println!("Connecting to {}", server_address);
    // Connect to the server
    let mut stream = TcpStream::connect(server_address).expect("Could not connect to server");

    // Read user input and send it to the server
    //vector to store server response

    //loop to read user input
    //filename
    let mut buffer = [0 as u8; 50];
    //read from tcp stream and send response ok
    stream.read(&mut buffer).unwrap();
    //print response
    println!("Receiving file: {}", String::from_utf8_lossy(&buffer[..]));
    //create &str from buffer
    let filnme = String::from_utf8_lossy(&buffer[..]);
    let filename = filnme.replace('\0', "");
    let file_to_return = filename.to_string();
    //send Ok to server
    stream.write(b"OK").unwrap();

    //read hash of file
    let mut buffer = [0 as u8; 64];
    //read from tcp stream and send response "OK"
    stream.read(&mut buffer).unwrap();
    //print response
    println!(
        "Verifying file with hash: {}",
        String::from_utf8_lossy(&buffer[..])
    );
    //create &str from buffer
    let hash = String::from_utf8_lossy(&buffer[..]);

    //send Ok to server
    stream.write(b"OK").unwrap();

    handle_client(stream, filename.to_string());

    let hash_file = calculate_file_hash(&filename).unwrap();
    println!("Hash of file: {}", hash_file);
    //compare hash
    if hash_file == hash {
        println!("Hashes match");
    } else {
        println!("Hashes don't match");
        //delete file
        fs::remove_file(filename).expect("Unable to delete file");
    }

    Ok(file_to_return)
}
fn get_first_hostname_ip() -> String {
    let my_local_ip = local_ip().unwrap();
    let ip_address = my_local_ip.to_string();
    ip_address
}

fn reduce_ip(ip_address: &str) -> String {
    //remove .
    let mut ip_address = ip_address.replace(".", "");
    //remove 192168
    ip_address = ip_address.replace("192168", "");
    ip_address
}

fn send_text_contents(stream: &mut TcpStream, filename: &str) -> io::Result<()> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        stream.write_all(line.as_bytes())?;
        stream.write_all(b"\n")?;
    }

    Ok(())
}
//read binary files
fn send_binary_contents(filename: &str, stream: &mut TcpStream) -> io::Result<()> {
    let mut file = File::open(filename)?;
    let mut buffer = [0; 512];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(());
        }
        stream.write_all(&buffer[..bytes_read])?;
    }
}

fn send(filename: &str) -> Result<(), io::Error> {
    //split filename into path and filename
    let path = std::path::PathBuf::from(&filename);
    let filename = path.file_name().unwrap().to_str().unwrap();



    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    //get_first_hostname_ip
    println!(
        "Ask recepient to run 'sharef r {}'",
        reduce_ip(get_first_hostname_ip().as_str())
    );

    //get file hash
    let file_hash = calculate_file_hash(path.to_str().unwrap()).unwrap();
    println!("Verify file with hash {}", file_hash);

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        //send filename and expect to receiv "ok", else exit
        stream.write_all(filename.as_bytes()).unwrap();
        println!("Sent filename, waiting for response");
        let mut buffer = [0; 2];
        stream.read_exact(&mut buffer).unwrap();
        if &buffer != b"OK" {
            println!("server did not respond with ok");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "server did not respond with ok on filename",
            ));
        }

        //send file hash and expect to receiv "ok", else exit
        stream.write_all(file_hash.as_bytes()).unwrap();
        println!("Sent file hash, waiting for response");
        let mut buffer = [0; 2];
        stream.read_exact(&mut buffer).unwrap();
        if &buffer != b"OK" {
            println!("server did not respond with ok");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "server did not respond with ok on file hash",
            ));
        }

        println!("sending file: {}", filename);

        //send file type as text if it fails send as binary
        if send_text_contents(&mut stream, &filename).is_err() {
            println!("sending as binary");
            send_binary_contents(&filename, &mut stream).unwrap();
        } else {
            println!("sending as text");
        }
        //send termination messag
        break;
    }

    Ok(())
}

fn create_zip(path: &str) -> String{
    let path = Path::new(path);
    //output is current working directory + filename + .zip
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

    Zip(opt).unwrap();

    return filename.to_string() + ".zip";
}

fn extract_zip(filename: &str){
    //remove .zip from filename
    let file = filename.replace(".zip", "");
    let file = Path::new(&file);

    let target = Path::new(&std::env::current_dir().unwrap())
        .join(file.file_name().unwrap());

    let archive: Vec<u8> = std::fs::read(filename).unwrap();

    println!("Extracting zip file: {}", filename);

    zip_extract::extract(Cursor::new(archive), &target, true).unwrap();
}

fn clean_up(filename: &str) {
    let filename = if !filename.ends_with(".zip") {
        filename.to_string() + ".zip"
    } else {
        filename.to_string()
    };

    let path = Path::new(&filename);

    //delete file if it exists
    if path.exists() {
        println!("Deleting file: {}", filename);
        std::fs::remove_file(filename).unwrap();
    }
}


fn usage() {
    println!("Usage: sharef [s | sf] <filename | folder> | [r | rf] <address>");
    println!("Example: sharef s file.txt");
    println!("Example: shared sf folder");
    println!("Example: shared r 3000 //When receiving a file");
    println!("Example: shared rf 3000 //When receiving a folder");
}


fn main() -> io::Result<()> {

    let args: Vec<String> = std::env::args().collect();
    //if no args print Usage
    if args.len() < 2 {
        usage();
        return Ok(());
    }

    if args[1] == "-h" || args[1] == "--help" {
        usage();
        return Ok(());
    }

    match args[1].as_str() {
        "s" => match send(&args[2]) {
            Ok(_) => println!("File sent"),
            Err(e) => println!("Error: {}", e),
        },
        "r" => match receive(args[2].to_string()) {
            Ok(_) => {
                println!("File received");
            }
            Err(e) => println!("Error: {}", e),
        },
         "rf" => match receive(args[2].to_string()) {
            Ok(filename) => {
                extract_zip(&filename);
                clean_up(&filename);
                println!("File received");
            }
            Err(e) => println!("Error: {}", e),
        },
        "sf" => {
            println!("{}",args[2]);
            match send(&create_zip(&args[2])) {
                Ok(_) => println!("File sent"),

                Err(e) => println!("Error: {}", e),

            }

            //get filename from path args[2]
            let filename = Path::new(&args[2]).file_name().unwrap().to_str().unwrap();
            clean_up(filename);
        }
        _ => {
            usage();
        }
    }

    Ok(())
}
