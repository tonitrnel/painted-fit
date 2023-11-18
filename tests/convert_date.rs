use chrono::{DateTime, Utc};
use fit::decoder::Decoder;
mod data;

#[test]
fn date_time_should_be_date() {
    let mut decoder = Decoder::new(&data::FIT_FILE_SHORT);
    let (_, messages) = decoder.decode().unwrap();
    assert_eq!(messages.get("file_id").map(|it| it.len()), Some(1));
    assert_eq!(
        messages
            .get("file_id")
            .and_then(|it| it[0].get("time_created"))
            .map(|it| it.to_string()),
        Some(
            DateTime::<Utc>::from_timestamp(1000000000 + 631065600, 0)
                .unwrap()
                .to_string()
        )
    )
}
