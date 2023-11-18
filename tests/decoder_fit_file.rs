use fit::decoder::Decoder;
use fit::error;
mod data;

#[test]
fn not_is_fit_file() {
    assert_eq!(
        Decoder::new(&[
            0x0E, 0x10, 0xD9, 0x07, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x91, 0x33,
            0x00, 0x00
        ])
        .decode(),
        Err(error::ErrorKind::InvalidFitFile)
    )
}

#[test]
fn have_1_message() {
    let mut decoder = Decoder::new(&data::FIT_FILE_SHORT);
    assert!(decoder.check_integrity());
    assert_eq!(
        decoder
            .decode()
            .ok()
            .and_then(|(_, it)| it.get("file_id").map(|it| it.len())),
        Some(1)
    )
}
