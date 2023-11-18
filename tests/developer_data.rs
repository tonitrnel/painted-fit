use fit::decoder::Decoder;
mod data;

#[test]
fn read_developer_data() {
    let buf = std::fs::read("tests/data/Activity.fit").unwrap();
    let mut decoder = Decoder::new(&buf);
    let (errors, messages) = decoder.decode().unwrap();
    assert_eq!(errors.len(), 0);
    assert_eq!(messages.get("record").map(|it| it.len()), Some(3601));
    println!("{:#?}", messages.keys())
}
