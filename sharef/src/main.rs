mod utils;


use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::{self, Write};
use std::io::{prelude::*, BufReader};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::Path;

//utils
use utils::{
    calculate_file_hash,
    reconstruct_ip,
    get_first_hostname_ip,
    reduce_ip,
    create_zip,
    extract_zip,
    clean_up,
};

fn handle_client(mut stream: TcpStream, filename: String) {
    let mut data = [0 as u8; 50];

    let mut file = File::create(filename).expect("create failed");

    println!("File created");

    while match stream.read(&mut data) {
        Ok(size) if size > 0 => {
            file.write_all(&data[0..size]).expect("write failed");
            true
        }
        Ok(_) => {
            println!("File received");
            println!("Checking checksum");

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

fn receive(small_ip: String) -> Result<String, String> {
    let server_address = reconstruct_ip(small_ip.to_string());

    println!("Connecting to {}", server_address);

    let mut stream = TcpStream::connect(server_address).expect("Could not connect to server");

    let mut buffer = [0 as u8; 50];

    stream.read(&mut buffer).unwrap();

    println!("Receiving file: {}", String::from_utf8_lossy(&buffer[..]));

    let filnme = String::from_utf8_lossy(&buffer[..]);
    let filename = filnme.replace('\0', "");
    let file_to_return = filename.to_string();

    stream.write(b"OK").unwrap();


    let mut buffer = [0 as u8; 64];

    stream.read(&mut buffer).unwrap();

    println!(
        "Verifying file with hash: {}",
        String::from_utf8_lossy(&buffer[..])
    );

    let hash = String::from_utf8_lossy(&buffer[..]);


    stream.write(b"OK").unwrap();

    handle_client(stream, filename.to_string());

    let hash_file = calculate_file_hash(&filename).unwrap();

    println!("Hash of file: {}", hash_file);

    //compare hash
    if hash_file == hash {
        println!("Hashes match");
    } else {
        println!("Hashes don't match");

        fs::remove_file(filename).expect("Unable to delete file");
    }

    Ok(file_to_return)
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

    let path = std::path::PathBuf::from(&filename);
    let filename = path.file_name().unwrap().to_str().unwrap();



    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();

    println!(
        "Ask recepient to run 'sharef r {}'",
        reduce_ip(get_first_hostname_ip().as_str())
    );


    let file_hash = calculate_file_hash(path.to_str().unwrap()).unwrap();
    println!("Verify file with hash {}", file_hash);

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

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


        if send_text_contents(&mut stream, &filename).is_err() {
            println!("sending as binary");
            send_binary_contents(&filename, &mut stream).unwrap();
        } else {
            println!("sending as text");
        }

        break;
    }

    Ok(())
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
