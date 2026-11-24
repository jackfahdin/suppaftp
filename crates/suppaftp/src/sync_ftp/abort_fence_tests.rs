use crate::{FtpStream, Status};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

fn check(replies: &'static [u8], valid: bool) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut control, _) = listener.accept().unwrap();
        control
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        control.write_all(b"220 ready\r\n").unwrap();
        let mut reader = BufReader::new(control.try_clone().unwrap());
        let mut data = None;
        for line in std::iter::from_fn(|| {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .ok()
                .filter(|n| *n > 0)
                .map(|_| line)
        }) {
            if line.starts_with("PASV") {
                let listener = TcpListener::bind("127.0.0.1:0").unwrap();
                let port = listener.local_addr().unwrap().port();
                control
                    .write_all(
                        format!("227 Passive (127,0,0,1,{},{})\r\n", port / 256, port % 256)
                            .as_bytes(),
                    )
                    .unwrap();
                data = Some(listener);
            } else if line.starts_with("LIST") {
                control.write_all(b"150 listing\r\n").unwrap();
                let (mut socket, _) = data.take().unwrap().accept().unwrap();
                socket.write_all(b"partial listing\r\n").unwrap();
                drop(socket);
            } else if line.starts_with("ABOR") {
                control.write_all(replies).unwrap();
            } else if line.starts_with("NOOP") {
                if control.write_all(b"200 fence\r\n").is_err() {
                    break;
                }
            } else if line.starts_with("PWD") {
                control.write_all(b"257 \"/after-abort\"\r\n").unwrap();
                break;
            } else {
                panic!("unexpected command");
            }
        }
    });
    let socket = TcpStream::connect(address).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut ftp = FtpStream::connect_with_stream(socket).unwrap();
    let (_, data) = ftp
        .custom_data_command("LIST /", &[Status::AboutToSend])
        .unwrap();
    let result = ftp.abort_with_fence(data);
    if valid {
        result.unwrap();
        assert_eq!(ftp.pwd().unwrap(), "/after-abort");
    } else {
        assert!(result.is_err());
    }
    drop(ftp);
    server.join().unwrap();
}

#[test]
fn abort_fence_consumes_normal_and_racing_completion_replies() {
    for replies in [
        b"226 aborted\r\n".as_slice(),
        b"426 aborted\r\n226 done\r\n",
        b"226 completed\r\n226 aborted\r\n",
        b"226 completed\r\n426 aborted\r\n226 done\r\n",
        b"225 no transfer\r\n",
    ] {
        check(replies, true);
    }
}

#[test]
fn abort_fence_rejects_unexpected_or_excessive_replies() {
    check(b"500 refused\r\n", false);
    check(b"200 invalid abort reply\r\n", false);
    check(b"226 a\r\n226 b\r\n226 c\r\n226 d\r\n", false);
}
