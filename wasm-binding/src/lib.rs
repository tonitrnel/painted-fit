use fit::decoder;
use fit::decoder::Messages;
use fit::error::ErrorKind;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const T_DECODE_RESULT: &'static str = r#"
export type Errors =  ReadonlyArray<{kind: string, message: string}>;
export type FieldValue = string | number | boolean;
export type Message = ReadonlyMap<string, FieldValue | FieldValue[]>;
export type Messages = ReadonlyMap<string, ReadonlyArray<Message>>;
export interface DecodeResult{
    errors: Errors
    messages: Messages
}
"#;

#[derive(Serialize)]
struct DecodeResult {
    errors: Vec<ErrorKind>,
    messages: Messages,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "DecodeResult")]
    pub type TDecodeResult;
}

#[wasm_bindgen]
pub fn decode(bytes: Vec<u8>) -> TDecodeResult {
    let mut decoder = decoder::Decoder::new(&bytes);
    let (errors, messages) = decoder.decode().unwrap();
    let result = DecodeResult { errors, messages };
    serde_wasm_bindgen::to_value(&result)
        .unwrap()
        .unchecked_into::<TDecodeResult>()
}
