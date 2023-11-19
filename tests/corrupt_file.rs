use fit::decoder::Decoder;
mod data;

#[test]
#[ignore]
fn test_read_past_mismatched_field() {
    let mut decoder = Decoder::new(&data::FIT_FILE_SHORT_WITH_WRONG_FIELD_DEF_SIZE);
    let (_errors, messages) = decoder.decode().unwrap();
    assert_eq!(messages.get("file_id").map(|it| it.len()), Some(1));
}
