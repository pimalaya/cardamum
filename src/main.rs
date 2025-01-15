use cardamum::std::IoConnector;

fn main() {
    let mut connector = IoConnector::new("localhost", 8001).unwrap();

    let contacts = connector
        .list_contacts("6fa928d4-e344-3021-1ad2-c652209ae251")
        .unwrap();

    println!("contacts: {contacts:#?}");
}
