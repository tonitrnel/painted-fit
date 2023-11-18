use fit::decoder::Decoder;
mod data;

#[test]
fn it_works() {
    let buf = std::fs::read("tests/data/WithGearChangeData.fit").unwrap();
    let mut decoder = Decoder::new(&buf);
    let (_, messages) = decoder.decode().unwrap();
    // assert_eq!(errors.len(), 0);
    assert_eq!(
        messages.get("event").map(|it| it
            .iter()
            .filter(|it| it.contains_key("gear_change_data"))
            .count()),
        Some(37)
    )
}

#[test]
fn test_sub_field_type_to_string_conversion() {
    let buf = std::fs::read("tests/data/WithGearChangeData.fit").unwrap();
    let mut decoder = Decoder::new(&buf);
    let (_, messages) = decoder.decode().unwrap();

    // assert_eq!(errors.len(), 0);

    assert!(messages
        .get("event")
        .unwrap()
        .iter()
        .filter(|it| it.contains_key("rider_position"))
        .map(|it| it
            .get("rider_position")
            .map(|it| matches!(it, fit::Value::String(_)))
            .unwrap())
        .all(|it| it))
}

#[test]
fn test_sub_fields_scale_offset() {
    let repeats = [
        &data::WORKOUT_800M_REPEATS_LITTLE_ENDIAN,
        &data::WORKOUT_800M_REPEATS_BIG_ENDIAN,
    ];
    for repeat in repeats {
        let mut decoder = Decoder::new(repeat);
        let (_, messages) = decoder.decode().unwrap();
        let distances: [f64; 4] = [4000.0, 800.0, 200.0, 1000.0];
        // assert_eq!(errors.len(), 0);
        for (idx, workout_step) in messages
            .get("workout_step")
            .map(|it| {
                it.iter()
                    .filter(|it| it.contains_key("duration_distance"))
                    .collect::<Vec<_>>()
            })
            .unwrap()
            .iter()
            .enumerate()
        {
            assert_eq!(
                workout_step.get("duration_distance"),
                Some(&fit::Value::Float64(distances[idx]))
            )
        }
    }
}
