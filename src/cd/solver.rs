use crate::cd::utils::{find_answers, generate_server_offset, make_id};

use chrono::prelude::*;
use rand::Rng;
use serde_json::json;

pub fn solve() -> String {
    let (server_offset, st, rst) = generate_server_offset();
    let current_time = Utc::now().timestamp_millis();

    let mut rng = rand::thread_rng();
    let runtime = rng.gen_range(1325.5..5525.5);
    let work_time = current_time - server_offset;
    let id = make_id();
    let (answers, _final_hash) = find_answers(&work_time.to_string(), &id);

    let multiplier: f64 = rng.gen_range(1.1..1.5);
    let adjusted_runtime: f64 = rng.gen_range(runtime..(runtime * multiplier));
    let duration = ((1000.0 * (adjusted_runtime - runtime)) / 1000.0).round() as u64;

    json!({
        "workTime": work_time,
        "id": id,
        "answers": answers,
        "duration": duration,
        "d": server_offset,
        "st": st,
        "rst": rst
    })
    .to_string()
}
