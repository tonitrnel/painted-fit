use fit::{decoder::Decoder, profile::types};
mod data;

#[test]
fn it_works() {
    let buf = std::fs::read("tests/data/Activity.fit").unwrap();
    let mut decoder = Decoder::new(&buf);
    let (errors, messages) = decoder.decode().unwrap();
    assert_eq!(errors.len(), 0);
    let records = messages.get("record").unwrap();
    for record in records {
        assert_eq!(record.get("enhanced_altitude"), record.get("altitude"));
        assert_eq!(record.get("enhanced_speed"), record.get("speed"));
    }
}

#[test]
fn test_hr_msg_event_timestamp12_expansion() {
    let buf = std::fs::read("tests/data/HrmPluginTestActivity.fit").unwrap();
    let mut decoder = Decoder::new(&buf);
    let (_errors, messages) = decoder.decode().unwrap();
    let mut i = 0;
    let hr_mesgs = messages.get("hr").unwrap();
    for hr_mesg in hr_mesgs {
        let event_timestamp = hr_mesg.get("event_timestamp");
        let event_timestamps = match event_timestamp {
            Some(fit::Value::Array(arr)) => arr,
            _ => break,
        };
        for event_timestamp in event_timestamps {
            assert_eq!(
                event_timestamp,
                &fit::Value::Float64(data::expand_hr_mesgs::COMPONENT_EXPANSION_OF_HR_MESSAGES[i])
            );
            i += 1;
        }
    }
}

#[test]
fn test_expansion_with_enum_components() {
    let mut decoder = Decoder::new(&data::FIT_FILE_MONITORING);
    let (_, messages) = decoder.decode().unwrap();
    let monitoring = messages.get("monitoring").unwrap();
    assert_eq!(
        monitoring[0].get("activity_type"),
        Some(&fit::Value::String(
            types::ActivityType::Sedentary.to_string()
        ))
    );
    assert_eq!(monitoring[0].get("intensity"), Some(&fit::Value::UInt8(3)));

    assert_eq!(
        monitoring[1].get("activity_type"),
        Some(&fit::Value::String(
            types::ActivityType::Generic.to_string()
        ))
    );
    assert_eq!(monitoring[1].get("intensity"), Some(&fit::Value::UInt8(0)));

    assert_eq!(
        monitoring[2].get("activity_type"),
        Some(&fit::Value::Enum(30))
    );
    assert_eq!(monitoring[2].get("intensity"), Some(&fit::Value::UInt8(6)));

    assert_eq!(monitoring[3].get("activity_type"), None);
    assert_eq!(monitoring[3].get("intensity"), None);
}
