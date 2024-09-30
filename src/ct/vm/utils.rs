const CHARSET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

pub fn get_vm_bytes(instructions: &str) -> Vec<i64> {
    let mut bytes = Vec::new();

    let mut m = 0;
    while m < instructions.len() {
        let mut h = 0;
        let mut l = 1;

        loop {
            let x = CHARSET.find(instructions.as_bytes()[m] as char).unwrap() as i64;
            m += 1;

            h += l * (x % 50 as i64);
            if x < 50 as i64 {
                bytes.push(h as i64);
                break;
            }

            h += 50 as i64 * l;
            l *= 12 as i64;
        }
    }

    bytes
}

fn decode_string(encoded: &[i64], index: &mut usize) -> String {
    *index += 1;
    let length = encoded[*index] as usize;
    *index += 1;

    (0..length)
        .map(|_| {
            let k = encoded[*index];
            *index += 1;
            let char_code = (k as u32 & 0xFFFFFFC0) | ((k * 39) & 63) as u32;
            char::from_u32(char_code).unwrap_or('\0')
        })
        .collect()
}

pub fn decode_vm_bytes(mut vm_bytes: Vec<i64>) -> (String, Vec<i64>) {
    let len = vm_bytes.len() as i64;
    let offset = (vm_bytes[len as usize - 1] ^ len) as usize;

    let f_len = vm_bytes[offset + 1] as usize + 2;
    let f: Vec<i64> = vm_bytes[offset..offset + f_len].to_vec();
    vm_bytes.drain(offset..offset + f_len);

    let decoded = decode_string(&f, &mut 0);
    (decoded, vm_bytes)
}
