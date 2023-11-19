use fit::decoder::Decoder;
use std::fs::File;
use std::io::Read;

fn main() {
    let mut fp = File::open("examples/Activity.fit").unwrap();
    let mut buffer = Vec::new();
    fp.read_to_end(&mut buffer).expect("Read file failed");
    let mut decoder = Decoder::new(&buffer);
    println!("is_fit = {}", Decoder::is_fit(&buffer));
    println!("check_integrity = {}", decoder.check_integrity());
    let (errors, messages) = decoder.decode().unwrap();
    let records = messages.get("record").unwrap();
    println!("errors = {:#?}", errors);
    println!("file id = {:#?}", messages.get("file_id").unwrap());
    println!("record size = {}", records.len());
    println!("record = {:#?}", records[0])
}
