use std::error::Error;

fn e(data: &[u8], t: usize) -> [u32; 2] {
    let r = 8 * t;
    [n(data, r), n(data, r + 4)]
}

fn t(key: &[u32; 4], v1: [u32; 2], v2: [u32; 2], output: &mut Vec<u8>, c: usize) -> [u32; 2] {
    let [t, n] = v1;
    let [r, o] = v2;
    let a = tea_round(key, [r ^ t, o ^ n]);
    store_u32_pair(output, c, a);
    a
}

fn n(data: &[u8], t: usize) -> u32 {
    ((data[t] as u32) << 24)
        | ((data[t + 1] as u32) << 16)
        | ((data[t + 2] as u32) << 8)
        | (data[t + 3] as u32)
}

fn tea_round(key: &[u32; 4], mut state: [u32; 2]) -> [u32; 2] {
    let mut sum = 0u32;
    for _ in 0..32 {
        state[0] = state[0].wrapping_add(
            (((state[1] << 4) ^ (state[1] >> 5)).wrapping_add(state[1]))
                ^ (sum.wrapping_add(key[sum as usize & 3])),
        );
        sum = sum.wrapping_add(0x9E3779B9);
        state[1] = state[1].wrapping_add(
            (((state[0] << 4) ^ (state[0] >> 5)).wrapping_add(state[0]))
                ^ (sum.wrapping_add(key[(sum >> 11) as usize & 3])),
        );
    }
    state
}

fn store_u32_pair(output: &mut Vec<u8>, index: usize, pair: [u32; 2]) {
    let o = 8 * index;
    output.extend_from_slice(&[
        (pair[o] >> 24) as u8,
        (pair[o + 1] >> 16) as u8,
        (pair[o + 2] >> 8) as u8,
        pair[o + 3] as u8,
        (pair[o + 4] >> 24) as u8,
        (pair[o + 5] >> 16) as u8,
        (pair[o + 6] >> 8) as u8,
        pair[o + 7] as u8,
    ]);
}

pub fn process_data(key: &[u8], iv: &[u8], input: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if key.len() != 16 {
        return Err("key must be 128-bit".into());
    }
    if iv.len() != 8 {
        return Err("iv must be 64-bit".into());
    }

    let c = [n(key, 0), n(key, 4), n(key, 8), n(key, 12)];
    let a = [n(iv, 0), n(iv, 4)];
    let mut s = vec![[0, input.len() as u32]];
    let u = (input.len() as f64 / 8.0).ceil() as usize;

    let mut l = 0;
    let mut d = 0;
    let mut f = Vec::new();
    let mut p = t(&c, [0, 0], a, &mut f, d);
    d += 1;

    while s.len() < 5 && l < u {
        s.push(e(input, l));
        l += 1;
    }

    let mut y = 0;
    while !s.is_empty() {
        y = ((y as i32 + p[0] as i32) % s.len() as i32) as usize;
        y += s.len();

        p = t(&c, p, s[y], &mut f, d);
        d += 1;
        if l < u {
            s[y] = e(input, l);
            l += 1;
        } else {
            s.remove(y);
        }
    }

    Ok(f)
}

pub fn string_to_bytes(s: &str) -> Vec<u8> {
    s.bytes().map(|b| b).collect()
}
