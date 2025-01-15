use std::env;

use cardamum::{
    carddav::sans_io::ListContactsFlow,
    tcp::{sans_io::Io as TcpIo, std::StdConnector},
};

fn main() {
    let host = env::var("HOST").unwrap_or(String::from("localhost"));
    println!("using host: {host:?}");

    let port = env::var("PORT").unwrap_or(String::from("8001"));
    let port = port.parse::<u16>().expect("should be an integer");
    println!("using port: {port:?}");

    // TCP I/O connector

    let mut tcp = StdConnector::connect(host, port).unwrap();

    // List CardDAV contacts

    let mut flow = ListContactsFlow::new("test", "6fa928d4-e344-3021-1ad2-c652209ae251");
    while let Some(io) = flow.next() {
        match io {
            TcpIo::Read => {
                tcp.read(&mut flow).unwrap();
            }
            TcpIo::Write => {
                tcp.write(&mut flow).unwrap();
            }
        }
    }

    let contacts = flow.take_contacts().unwrap();
    println!("contacts: {contacts:#?}");
}
