use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct RandId {
    id: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum PairingInfomation {
    #[serde(rename = "rand-id")]
    RandId(RandId),
}

fn main() {
    let json = r#"{"type": "rand-id", "id": "1234"}"#;
    match serde_json::from_str::<PairingInfomation>(json) {
        Ok(v) => println!("Success: {:?}", v),
        Err(e) => println!("Error: {}", e),
    }
}
