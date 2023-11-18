use fit::decoder::Decoder;
use fit::Value;

#[test]
fn should_be_applied() {
    let buf = std::fs::read("tests/data/Activity.fit").unwrap();
    let mut decoder = Decoder::new(&buf);
    let (_, messages) = decoder.decode().unwrap();
    assert_eq!(
        messages.get("record").and_then(|it| it[0].get("altitude")),
        Some(&Value::Float64(127.0))
    )
}
#[test]
fn should_be_applied_to_arrays() {
    let buf = std::fs::read("tests/data/WithGearChangeData.fit").unwrap();
    let mut decoder = Decoder::new(&buf);
    let (_, messages) = decoder.decode().unwrap();
    let left_power_phase = messages
        .get("record")
        .and_then(|it| it[28].get("left_power_phase"));
    let left_power_phase_peak = messages
        .get("record")
        .and_then(|it| it[28].get("left_power_phase_peak"));

    let right_power_phase = messages
        .get("record")
        .and_then(|it| it[28].get("right_power_phase"));
    let right_power_phase_peak = messages
        .get("record")
        .and_then(|it| it[28].get("right_power_phase_peak"));

    assert_eq!(
        left_power_phase,
        Some(&Value::Array(vec![
            Value::Float64(337.5000052734376),
            Value::Float64(199.68750312011724)
        ]))
    );
    assert_eq!(
        left_power_phase_peak,
        Some(&Value::Array(vec![
            Value::Float64(75.93750118652346),
            Value::Float64(104.0625016259766)
        ]))
    );

    assert_eq!(
        right_power_phase,
        Some(&Value::Array(vec![
            Value::Float64(7.031250109863283),
            Value::Float64(205.31250320800785)
        ]))
    );
    assert_eq!(
        right_power_phase_peak,
        Some(&Value::Array(vec![
            Value::Float64(70.31250109863284),
            Value::Float64(106.8750016699219)
        ]))
    );
}
