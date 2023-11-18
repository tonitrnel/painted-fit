use fit::decoder::Decoder;
mod data;

#[test]
fn should_converted() {
    let mut decoder = Decoder::new(&data::FIT_FILE_SHORT);
    let (_, messages) = decoder.decode().unwrap();
    assert_eq!(messages.get("file_id").map(|it| it.len()), Some(1));
    assert_eq!(
        messages
            .get("file_id")
            .and_then(|it| it[0].get("type"))
            .map(|it| it.to_string()),
        Some(String::from("activity"))
    )
}
