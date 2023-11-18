use fit::decoder::Decoder;
mod data;

#[test]
fn have_2_message() {
    let mut decoder = Decoder::new(&data::FIT_FILE_CHAINED);
    let result = decoder.decode();
    assert!(result.is_ok());
    let (_, messages) = result.unwrap();
    assert_eq!(messages.get("file_id").map(|it| it.len()), Some(2))
}

#[test]
fn have_1_message() {
    let mut decoder = Decoder::new(&data::FIT_FILE_CHAINED_WEIRD_VIVOKI);
    let result = decoder.decode();
    assert!(result.is_ok());
    let (_, messages) = result.unwrap();
    assert_eq!(messages.get("file_id").map(|it| it.len()), Some(1))
}
