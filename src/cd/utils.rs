use chrono::prelude::*;
use rand::Rng;
use sha2::{Digest, Sha256};

pub fn generate_server_offset() -> (i64, i64, i64) {
    let tmstp = Utc::now().timestamp_millis();
    let tmstp2 = tmstp + 1400 + rand::thread_rng().gen_range(0..1300);

    ((tmstp2 - tmstp), tmstp, tmstp2)
}

pub fn get_hash_difficulty(hash: &str) -> f64 {
    let base: i64 = 0x10000000000000 as i64;
    let denominator = u64::from_str_radix(&hash[..13], 16).unwrap_or(0) as f64 + 1.0;
    base as f64 / denominator
}

pub fn make_id() -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "0123456789abcdef".chars().collect();

    (0..32).map(|_| chars[rng.gen_range(0..16)]).collect()
}

pub fn find_answers(start_value: &str, id_hash: &str) -> (Vec<u64>, String) {
    let mut answers = Vec::new();
    let mut current_hash = hex::encode(Sha256::digest(
        format!("tp-v2-input, {}, {}", start_value, id_hash).as_bytes(),
    ));

    for _ in 0..2 {
        let mut nonce = 1u64;
        loop {
            let new_hash = hex::encode(Sha256::digest(
                format!("{}, {}", nonce, current_hash).as_bytes(),
            ));
            if get_hash_difficulty(&new_hash) >= 5.0 {
                answers.push(nonce);
                current_hash = new_hash;
                break;
            }
            nonce += 1;
        }
    }

    (answers, current_hash)
}
