use std::io::{self, Write};
use std::net::{TcpStream, Shutdown};
use std::io::Read;
use std::fs::File;

fn reconstruct_ip(small_ip: String) -> String{
    //example 3052
    //convert to 192.168.30.52 192.168 is the network address
    let ip = String::from("192.168.");
    let small_ip = small_ip;
    let mut ip = ip + &small_ip[0..2] + "." + &small_ip[2..4];
    //add port 7878
    ip = ip + ":7878";
    ip
}
fn handle_client(mut stream: TcpStream, filename :String) {
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
        },
        Ok(_) => {
            println!("File received");
            println!("Checking checksum");
            //check checksum
            false
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn main() -> io::Result<()> {
    //get small ip from command line
    let args: Vec<String> = std::env::args().collect();
    let small_ip = &args[1];
    //reconstruct ip
    let server_address = reconstruct_ip(small_ip.to_string());

    println!("Connecting to {}", server_address);
    // Connect to the server
    let mut stream = TcpStream::connect(server_address)?;

    // Read user input and send it to the server
    //vector to store server response

    //loop to read user input
    //filename
    let mut buffer = [0 as u8; 50];
    //read from tcp stream and send response ok
    stream.read(&mut buffer).unwrap();
    //print response
    println!("Server response: {}", String::from_utf8_lossy(&buffer[..]));
    //create &str from buffer
    let filnme = String::from_utf8_lossy(&buffer[..]);
    let filename = filnme.replace('\0',"");
    //send Ok to server
    stream.write(b"OK").unwrap();
    handle_client(stream, filename.to_string());

    Ok(())
}
