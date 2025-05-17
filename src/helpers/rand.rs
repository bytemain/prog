use random_string::generate;

pub fn get_random_string(n: usize) -> String {
    // generate random string with fixed length
    let seed = "0123456789abcdefghijklmnopqrstuvwxyz";

    generate(n, seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_random_string() {
        let n = 10;
        let result = get_random_string(n);
        println!("random string {}", result);
        assert_eq!(result.len(), { n });
    }
}
