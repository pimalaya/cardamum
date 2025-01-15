use std::env;

use cardamum::{
    carddav::sans_io::{CurrentUserPrincipalFlow, ListContactsFlow},
    tcp::{sans_io::Io as TcpIo, std::StdConnector},
};

fn main() {
    let host = env::var("HOST").unwrap_or(String::from("localhost"));
    println!("using host: {host:?}");

    let port = env::var("PORT").unwrap_or(String::from("8001"));
    let port = port.parse::<u16>().expect("should be an integer");
    println!("using port: {port:?}");

    let user = env::var("USER").unwrap_or(String::from("test"));
    println!("using user: {user:?}");

    let password = env::var("PASSWORD").unwrap_or(String::from("test"));
    println!("using password: {password:?}");

    // Current user principal

    // NOTE: ideally, this should be needed once in order to re-use
    // the connection. It depends on the HTTP protocol returned by the
    // server.
    let mut tcp = StdConnector::connect(&host, port).unwrap();

    let mut flow = CurrentUserPrincipalFlow::new("test");
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

    let output = flow.output().unwrap();
    println!("current user principal: {output:#?}");

    // List CardDAV contacts

    let mut tcp = StdConnector::connect(&host, port).unwrap();
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

    let output = flow.output().unwrap();
    println!("contacts: {output:#?}");
}
