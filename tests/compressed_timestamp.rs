use fit::decoder::Decoder;
mod data;
#[test]
#[ignore]
// test data is not available
fn should_be_compressed() {
    let mut decoder = Decoder::new(&data::FIT_FILE_SHORT_COMPRESSED_TIMESTAMP);
    let (errors, messages) = decoder.decode().unwrap();
    println!("errors = {errors:?}\nmessages={messages:?}");
}
