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

pub fn get_random_string(n: i32) -> String {
    // generate random string with fixed length
    let seed = decimal_to_base(random_seed() as usize, "0123456789abcdefghijklmnopqrstuvwxyz");

    let mut result = String::new();
    for c in seed.chars() {
        if result.len() == n as usize {
            break;
        } else {
            result.push(c);
        }
    }
    result
}

pub fn decimal_to_base(mut nbr: usize, to_base: &str) -> String {
    if nbr == 0 {
        return to_base.chars().next().unwrap().into();
    }
    let base_length = to_base.chars().count();
    let mut result = String::new();
    while nbr > 0 {
        result.push(to_base.chars().nth(nbr % base_length).unwrap());
        nbr /= base_length;
    }

    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_seed() {
        let seed = random_seed();
        println!("seed {}", seed);
        // check if the seed is greater than 0
        assert!(seed > 0);
    }

    #[test]
    fn test_gen_n_number() {
        let n = 10;
        let result = gen_n_number(n);
        println!("{}", result);
        assert_eq!(result.len(), n as usize);
        assert!(result.chars().all(char::is_numeric));
    }

    #[test]
    fn test_get_random_string() {
        let n = 10;
        let result = get_random_string(n);
        println!("random string {}", result);
        assert_eq!(result.len(), n as usize);
    }

    #[test]
    fn test_decimal_to_base() {
        let result = decimal_to_base(10, "01");
        assert_eq!(result, "1010");

        let result = decimal_to_base(10, "0123456789");
        assert_eq!(result, "10");

        let result = decimal_to_base(10, "0123456789ABCDEF");
        assert_eq!(result, "A");

        let result = decimal_to_base(1241512341234, "0123456789abcdefghijklmnopqrstuvwxyz");
        assert_eq!(result, "fucce5f6");
    }
}
