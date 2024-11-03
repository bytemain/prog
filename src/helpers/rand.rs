use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

pub fn random_seed() -> u64 {
    RandomState::new().build_hasher().finish()
}

pub fn gen_n_number(n: i32) -> String {
    // generate random number with fixed length
    let seed = random_seed().to_string();
    String::from(&seed[..n as usize])
}