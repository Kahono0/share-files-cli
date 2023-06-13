use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use std::fs::File;
use std::io;
use local_ip_address::local_ip;

fn get_first_hostname_ip() -> String{
    let my_local_ip = local_ip().unwrap();
    let mut ip_address = my_local_ip.to_string();
    ip_address
}

fn reduce_ip(ip_address: &str) -> String{
    println!("{}", ip_address);
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


fn main() {
    //get filename from command line
    let filename = std::env::args().nth(1).expect("no filename given");

    //split filename into path and filename
    let path = std::path::PathBuf::from(&filename);
    let filename = path.file_name().unwrap().to_str().unwrap();

    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();

    //get_first_hostname_ip
    println!("{}",reduce_ip(get_first_hostname_ip().as_str()));

    //println!("listening on: {}", ip_address);

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        //send filename and expect to receiv "ok", else exit
        stream.write_all(filename.as_bytes()).unwrap();
        println!("Sent filename, waiting for response");
        let mut buffer = [0; 2];
        stream.read_exact(&mut buffer).unwrap();
        if &buffer != b"OK" {
            println!("server did not respond with ok");
            return;
        }

        println!("sending file: {}", filename);

        //send file type as text if it fails send as binary
        if send_text_contents(&mut stream, &filename).is_err() {

            println!("sending as binary");
            send_binary_contents(&filename, &mut stream).unwrap();
        }else{
            println!("sending as text");
        }

        println!("file sent");

        //send termination message
        stream.write_all(b"END").unwrap();

        return;
    }
}
