use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use super::{ImplFtpStream, TlsConnector, TlsStream};
use crate::FtpResult;

#[derive(Debug)]
struct TestStream(TcpStream);

impl TlsStream for TestStream {
    type InnerStream = TcpStream;

    fn tcp_stream(self) -> FtpResult<TcpStream> {
        Ok(self.0)
    }

    fn get_ref(&self) -> &TcpStream {
        &self.0
    }

    fn mut_ref(&mut self) -> &mut TcpStream {
        &mut self.0
    }
}

#[derive(Debug)]
struct TestConnector;

impl TlsConnector for TestConnector {
    type Stream = TestStream;

    fn connect(&self, domain: &str, socket: TcpStream) -> FtpResult<TestStream> {
        assert_eq!(domain, "localhost");
        assert_eq!(
            socket.read_timeout().unwrap(),
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            socket.write_timeout().unwrap(),
            Some(Duration::from_millis(100))
        );
        Ok(TestStream(socket))
    }
}

#[test]
fn implicit_ftps_keeps_socket_options_before_tls_and_greeting() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let socket = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (mut server, _) = listener.accept().unwrap();
    server.write_all(b"220 ready\r\n").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    socket
        .set_write_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    let ftp =
        ImplFtpStream::connect_secure_implicit_with_stream(socket, TestConnector, "localhost")
            .unwrap();
    assert_eq!(ftp.get_welcome_msg(), Some("220 ready"));
}

#[test]
fn implicit_ftps_greeting_respects_the_supplied_socket_deadline() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let socket = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (mut server, _) = listener.accept().unwrap();
    socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    socket
        .set_write_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    assert!(
        ImplFtpStream::connect_secure_implicit_with_stream(socket, TestConnector, "localhost")
            .is_err()
    );
    server
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    assert_eq!(server.read(&mut [0]).unwrap(), 0);
}
