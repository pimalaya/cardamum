use std::{env, process::Command};

use cardamum::{
    carddav::sans_io::{
        AddressbookHomeSetFlow, Addressbooks, CurrentUserPrincipalFlow, ListContactsFlow,
    },
    tcp::{sans_io::Io as TcpIo, std::Connector},
};

fn main() {
    let host = env::var("HOST").unwrap_or(String::from("localhost"));
    println!("using host: {host:?}");

    let port = env::var("PORT").unwrap_or(String::from("8001"));
    let port = port.parse::<u16>().expect("should be an integer");
    println!("using port: {port:?}");

    let version = env::var("VERSION").unwrap_or(String::from("1.1"));
    println!("using HTTP version: {version:?}");

    let user = env::var("USER").unwrap_or(String::from("test"));
    println!("using user: {user:?}");

    let password = env::var("PASSWORD_COMMAND").expect("missing env PASSWORD_COMMAND");
    println!("using password command: {password:?}");
    let mut args = password.split_whitespace();
    let program = args.next().unwrap();
    let password = Command::new(program).args(args).output().unwrap().stdout;
    let password = String::from_utf8_lossy(password.trim_ascii());

    println!("starting the example");

    // Current user principal

    // NOTE: ideally, this should be needed once in order to re-use
    // the connection. It depends on the HTTP protocol returned by the
    // server.
    let mut tcp = Connector::connect(&host, port).unwrap();
    let mut flow = CurrentUserPrincipalFlow::new("/", &version, &user, &password);

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

    let output = flow.output().unwrap().unwrap();

    println!("current user principal output: {output:#?}");

    let current_user_principal_url = output
        .responses
        .into_iter()
        .next()
        .unwrap()
        .propstats
        .into_iter()
        .next()
        .unwrap()
        .prop
        .current_user_principal
        .href
        .value;

    println!("current user principal: {current_user_principal_url:?}");

    // Addressbook home set

    let mut tcp = Connector::connect(&host, port).unwrap();
    let mut flow =
        AddressbookHomeSetFlow::new(current_user_principal_url, &version, &user, &password);

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

    let output = flow.output().unwrap().unwrap();

    println!("addressbook home set output: {output:#?}");

    let addressbook_home_set_url = output
        .responses
        .into_iter()
        .next()
        .unwrap()
        .propstats
        .into_iter()
        .next()
        .unwrap()
        .prop
        .addressbook_home_set
        .href
        .value;

    println!("addressbook home set: {addressbook_home_set_url:?}");

    // Addressbooks

    let mut tcp = Connector::connect(&host, port).unwrap();
    let mut flow = Addressbooks::new(addressbook_home_set_url, &version, &user, &password);

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

    let output = flow.output().unwrap().unwrap();

    println!("addressbooks output: {output:#?}");

    let addressbook_hrefs = output.get_addressbook_hrefs().collect::<Vec<_>>();

    println!(
        "found {} addressbooks: {addressbook_hrefs:#?}",
        addressbook_hrefs.len()
    );

    // List CardDAV contacts

    let addressbook_href = addressbook_hrefs.into_iter().next().unwrap();

    let mut tcp = Connector::connect(&host, port).unwrap();
    let mut flow = ListContactsFlow::new(addressbook_href, &version, &user, &password);

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

    println!("contacts output: {output:#?}");
}
