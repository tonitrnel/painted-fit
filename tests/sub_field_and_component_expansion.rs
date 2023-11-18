use fit::decoder::Decoder;
use serde::{Deserialize, Serialize};
mod data;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct GearChangeData {
    timestamp: u32,
    rear_gear_num: u8,
    rear_gear: u8,
    front_gear_num: u8,
    front_gear: Option<u8>,
    data: u32,
    gear_change_data: u32,
}

#[test]
fn it_works() {
    let buf = std::fs::read("tests/data/WithGearChangeData.fit").unwrap();
    let mut decoder = Decoder::new(&buf);
    let (_errors, messages) = decoder.decode().unwrap();
    let test_data: Vec<GearChangeData> = {
        let test_buf = std::fs::read("tests/data/gear_change_data.json").unwrap();
        serde_json::from_slice(&test_buf).unwrap()
    };
    let mesgs = messages
        .get("event")
        .map(|it| {
            it.iter()
                .filter(|it| it.contains_key("gear_change_data"))
                .collect::<Vec<_>>()
        })
        .unwrap();
    assert_eq!(mesgs.len(), test_data.len());
    for (idx, mesg) in mesgs.iter().enumerate() {
        println!("{}", serde_json::to_string(mesg).unwrap());
        if idx > 10 {
            break;
        }
        assert_eq!(
            mesg.get("gear_change_data"),
            Some(&fit::Value::UInt32(test_data[idx].gear_change_data))
        );
        assert_eq!(
            mesg.get("data"),
            Some(&fit::Value::UInt32(test_data[idx].data))
        );
        assert_eq!(
            mesg.get("front_gear_num"),
            Some(&fit::Value::UInt8z(test_data[idx].front_gear_num))
        );
        assert_eq!(
            mesg.get("front_gear"),
            Some(&fit::Value::UInt8z(test_data[idx].front_gear.unwrap_or(0)))
        );
        assert_eq!(
            mesg.get("rear_gear_num"),
            Some(&fit::Value::UInt8z(test_data[idx].rear_gear_num))
        );
        assert_eq!(
            mesg.get("rear_gear"),
            Some(&fit::Value::UInt8z(test_data[idx].rear_gear))
        );
    }
}
