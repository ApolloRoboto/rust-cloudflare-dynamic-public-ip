use rand::Rng;

pub fn generate_random_string(length: usize) -> String {
    let charset = b"abcdef0123456789";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

