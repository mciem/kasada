use unicode_segmentation::UnicodeSegmentation;

pub fn get_vm_bytes(instructions: &str, charset: &str) -> Vec<i64> {
    let charset_index = {
        let mut arr = [62_usize; 256];
        charset.graphemes(true).enumerate().for_each(|(i, g)| {
            if let Some(&c) = g.as_bytes().first() {
                arr[c as usize] = i;
            }
        });
        arr
    };

    let mut result = Vec::with_capacity(instructions.len() / 3);
    let mut iter = instructions.graphemes(true).map(|g| g.as_bytes()[0]);
    while let Some(mut byte) = iter.next() {
        let mut h = 0;
        let mut l = 1;

        loop {
            let x = charset_index[byte as usize] as i64;

            h += l * (x % 50);

            if x < 50 {
                result.push(h);
                break;
            }

            h += 50 * l;
            l *= 12;

            byte = iter.next().unwrap();
        }
    }

    result
}

pub fn decode_vm_bytes(mut vm_bytes: Vec<i64>) -> (String, Vec<i64>) {
    let len = vm_bytes.len() as i64;
    let offset = (vm_bytes[len as usize - 1] ^ (len + 4)) as usize;
    let f_len = vm_bytes[offset + 1] as usize + 2;

    let f = &vm_bytes[offset..offset + f_len];
    let decoded = decode_string(f, &mut 0);

    vm_bytes.copy_within(offset + f_len.., offset);
    vm_bytes.truncate(vm_bytes.len() - f_len);

    (decoded, vm_bytes)
}

fn decode_string(encoded: &[i64], index: &mut usize) -> String {
    *index += 1;
    let length = encoded[*index] as usize;
    *index += 1;

    let mut result = String::with_capacity(length);
    for _ in 0..length {
        let k = encoded[*index];
        *index += 1;
        let char_code = (k as u32 & 0xFFFFFFC0) | ((k * 59) & 63) as u32;
        if let Some(c) = char::from_u32(char_code) {
            result.push(c);
        } else {
            result.push('\0');
        }
    }

    result
}
