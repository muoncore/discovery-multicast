extern crate libc;
extern crate net2;
extern crate mio;
extern crate bytes;
extern crate std;
//#[macro_use]
//extern crate log;

use std::fmt;
use self::libc::c_char;
use std::ffi::CString;
use std::thread;
use std::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;
use std::ops::Drop;
use std::str;
use self::bytes::{Buf, MutBuf, RingBuf, SliceBuf};
use self::mio::udp::*;
use self::mio::*;
use self::mio::net::UdpSocket;
use self::mio::deprecated::{EventLoop, Handler};
use self::net2::UdpBuilder;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub struct UdpHandler {
    rx: UdpSocket,
    rx_buf: RingBuf,
    localhost: IpAddr
}

impl UdpHandler {
    fn new(rx: UdpSocket) -> UdpHandler {
        let sock = UdpSocket::bind(&"0.0.0.0:0".parse().unwrap()).unwrap();
        UdpHandler {
            rx: rx,
            rx_buf: RingBuf::new(1024),
            localhost: sock.local_addr().unwrap().ip()
        }
    }

    fn handle_read(&mut self, event_loop: &mut EventLoop<UdpHandler>, token: Token, _: Ready) {
        match token {
            LISTENER => {
                debug!("We are receiving a datagram now...");
                unsafe {
                    let dat = self.rx.recv_from(self.rx_buf.mut_bytes());
                    if (dat.is_ok()) {
                        let val = dat.unwrap();
                        println!("RECEIVED DATA {}", str::from_utf8(self.rx_buf.mut_bytes()).unwrap())

                    }
                }
            },
            _ => ()
        }
    }
}

impl Handler for UdpHandler {
    type Timeout = usize;
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<UdpHandler>, token: Token, events: Ready) {
        if events.is_readable() {
            println!("readables");
            self.handle_read(event_loop, token, events);
        }
    }
}


#[repr(C)]
pub struct ServiceDescriptor {
    pub identifier: *mut c_char,
    pub tags: Vec<&'static str>,
    pub codecs: Vec<&'static str>,
    pub connection_urls: Vec<&'static str>
}

impl ServiceDescriptor {
    pub fn create(identifier: &str) -> ServiceDescriptor {
        ServiceDescriptor {
            identifier: (CString::new("AWESOME").unwrap().into_raw()),
            tags: vec!["h", "b"],
            codecs: vec!["h", "b"],
            connection_urls: vec!["h", "b"],
        }
    }
    pub fn get_identifier(&self) -> String {
        unsafe {
            return CString::from_raw(self.identifier).into_string().unwrap();
        }
    }
}


pub struct MulticastData {
    pub name: &'static str,
}

#[repr(C)]
pub struct MulticastDiscovery {
    pub name: &'static str,
    pub lock: Arc<Mutex<MulticastData>>
}

pub fn run() -> MulticastDiscovery{

    let data = Arc::new(Mutex::new(MulticastData { name: "awesome"}));

    let multi = MulticastDiscovery::create(data);
    multi
}

impl MulticastDiscovery {
    pub fn create(data: Arc<Mutex<MulticastData>>) -> MulticastDiscovery {

        let threaddata = data.clone();
        let senddata = data.clone();

        let address: SocketAddr = "0.0.0.0:7776".parse().unwrap();

        let builder = UdpBuilder::new_v4().unwrap();
        builder.reuse_address(true).unwrap();
        let sock = builder.bind(&address).unwrap();

        let rx = UdpSocket::from_socket(sock).unwrap();
        let addr = rx.local_addr().unwrap();

        //sender
        thread::spawn(move || {
            let any = "0.0.0.0:0".parse().unwrap();
            let tx = UdpSocket::bind(&any).unwrap();

            let localhost = tx.local_addr().unwrap();
            loop {
                {
                    let dat = senddata.lock().unwrap();
                    let msg = format!("{} {}", dat.name, address);
                    let mut buf = SliceBuf::wrap(msg.as_bytes());

                    println!("Sending....{}", addr);
                    let cnt = tx.send_to(buf.bytes(), &"227.1.1.100:7776".parse().unwrap())
                        .unwrap();
                }
//                buf.advance(cnt);
                thread::sleep(Duration::from_millis(500));
            }
        });

        //receiver
        thread::spawn(move || {
            const LISTENER: Token = Token(0);

            let mut event_loop = EventLoop::new().unwrap();


            println!("Joining group 227.1.1.100");
            let any = "0.0.0.0".parse().unwrap();
            rx.join_multicast_v4(&"227.1.1.100".parse().unwrap(), &any).unwrap();

            println!("Registering LISTENER");
            event_loop.register(&rx, LISTENER, Ready::readable(), PollOpt::edge()).unwrap();

            println!("Starting event loop to test with...");
            event_loop.run(&mut UdpHandler::new(rx)).unwrap();

        });

        return MulticastDiscovery { lock:data, name:"Happy" };
    }

    pub fn on_ready<F>(& mut self, arg: F)
        where F: Fn() {
        println!("CALLED ATTACHED METHOD");
//        println!("CALLED A ATTACHED METHOD!");
        arg();
    }
    pub fn advertise_local_service(& mut self, descriptor: &ServiceDescriptor) {
        println!("Got 0, {}", self.name);
        let data = self.lock.clone();
        println!("Got 1");
        let mut dat = data.lock().unwrap();
        println!("Got 2");
        dat.name = "HELLO WORLD";
        println!("Advertising a local instance {}", descriptor.get_identifier());
    }

    pub fn shutdown(& mut self) {
        println!("Internal shutdown has been called");

    }

    pub fn get_known_services(& mut self) /* TODO -> [ServiceDescriptor] */ {

//        []
    }
}