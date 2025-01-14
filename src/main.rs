use cardamum::std::IoConnector;

fn main() {
    let mut connector = IoConnector::new("localhost", 8001).unwrap();

    let contacts = connector.list_contacts().unwrap();

    println!("contacts: {contacts:#?}");
}
