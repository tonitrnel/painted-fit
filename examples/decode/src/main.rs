use fit::decoder::Decoder;
use std::fs::File;
use std::io::Read;

fn main() {
    let mut fp = File::open("examples/Activity.fit").unwrap();
    let mut buffer = Vec::new();
    fp.read_to_end(&mut buffer).expect("Read file failed");
    let mut decoder = Decoder::new(&buffer);
    let header = decoder.read_file_header();
    println!("is_fit = {}", Decoder::is_fit(&buffer));
    println!("check_integrity = {}", decoder.check_integrity(&header));
    println!("header = {:#?}", header);
    let messages = decoder.read();
    let records = messages.get("record").unwrap();
    println!("record size = {}", records.len());
    println!("record = {:#?}", records[0])
    // println!("{:#?}", messages);
    // for (name, records) in messages {
    //     println!("name = {name}");
    //     println!("messages = {:?}", records);
    //     println!("===================================================");
    // }
}
