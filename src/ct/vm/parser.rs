pub fn parse(bytes: Vec<i64>, decoded: &str) -> Vec<String> {
    let chars = decoded.chars().collect::<Vec<char>>();

    let mut i = 1;
    let mut result = Vec::new();

    while i < bytes.len() {
        let opcode = bytes[i];
        i += 1;

        if opcode == 32 {
            i += 2;

            let x = bytes[i] as usize;
            i += 1;

            let y = bytes[i] as usize;
            i += 1;

            let word: String = (x..(x + y)).map(|j| chars[bytes[j] as usize]).collect();

            result.push(word);
        }
    }

    result
}
