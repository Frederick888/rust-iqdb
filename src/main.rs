extern crate iqdb;

fn main() {
    let url = ::std::env::args().nth(1).expect("Expect 1 image URL");
    let services = iqdb::available_services().unwrap();
    let matches = iqdb::search_by_url(&url, &services).unwrap();
    matches.iter().for_each(|m| println!("{:?}", m));
}
