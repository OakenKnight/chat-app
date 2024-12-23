use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:8080";
const MSG_SIZE: usize = 32;

fn main() {
    println!("Enter your nickname: ");
    let mut nickname = String::new();
    io::stdin().read_line(&mut nickname).expect("Failed to read nickname");
    let nickname = nickname.trim().to_string();
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client
        .set_nonblocking(true)
        .expect("Failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<String>();
    // let unmoved_nickname = nickname.clone();
    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];

        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff
                    .into_iter()
                    .take_while(|&x| x != 0)
                    .collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid UTF-8 message");
                println!("{}", msg);
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server was severed");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let formatmsg = format!("{}: {}", nickname.clone(), msg);
                let mut buff = formatmsg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client
                    .write_all(&buff)
                    .expect("Writing message to socket failed");
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a message: ");
    loop {
        let mut buff = String::new();

        io::stdin()
            .read_line(&mut buff)
            .expect("Failed to read from stdin");
        let msg = buff.trim().to_string();
        if msg == ":quit" || tx.send(msg).is_err() {
            break;
        }
    }
    println!("Bye!");
}
