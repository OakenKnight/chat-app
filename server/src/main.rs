use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use chrono::Local;

const LOCAL: &str = "127.0.0.1:8080";
const MSG_SIZE: usize = 32;

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind!");
    server
        .set_nonblocking(true)
        .expect("Failed to initialize non-blocking");

    let mut clients = vec![]; // Vec of (SocketAddr, TcpStream)
    let (tx, rx) = mpsc::channel::<(SocketAddr, String)>(); // Include sender's address with the message

    println!("Server running on {}", LOCAL);

    loop {
        // Accept new clients
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);
            let tx = tx.clone();
            clients.push((addr, socket.try_clone().expect("Failed to clone client")));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff
                            .into_iter()
                            .take_while(|&x| x != 0)
                            .collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid UTF-8 message");
    
                        // Format the date and time as a string
                        let timestamp = Local::now().format("%H:%M:%S").to_string();
                        println!("{addr} | {timestamp} | {msg}");
                        tx.send((addr, msg)).expect("Failed to send message to channel");
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with {}", addr);
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(100));
            });
        }

        // Broadcast messages to all clients except the sender
        if let Ok((sender_addr, msg)) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|(addr, mut client)| {
                    if addr != sender_addr {
                        let mut buff = msg.clone().into_bytes();
                        buff.resize(MSG_SIZE, 0);
                        client.write_all(&buff).map(|_| (addr, client)).ok()
                    } else {
                        Some((addr, client)) // Keep the sender client in the list
                    }
                })
                .collect::<Vec<_>>();
        }

        thread::sleep(Duration::from_millis(100));
    }
}