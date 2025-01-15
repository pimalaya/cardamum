use cardamum::std::IoConnector;

fn main() {
    let host = env::var("HOST").unwrap_or(String::from("localhost"));
    println!("using host: {host:?}");

    let port = env::var("PORT").unwrap_or(String::from("8001"));
    let port = port.parse::<usize>().expect("should be an integer");
    println!("using port: {port:?}");

    let mut connector = IoConnector::new(host, port).unwrap();

    let contacts = connector
        .list_contacts("6fa928d4-e344-3021-1ad2-c652209ae251")
        .unwrap();

    println!("contacts: {contacts:#?}");
}
