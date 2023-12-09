pub fn format_number(n: u64) -> String {
    // Check if number is >= 1000
    if n >= 1000 {
        let k = n / 1000;
        let remainder = n % 1000;

        // Check if remainder exists, add "." if it does
        if remainder > 0 {
            // calculate how many digits are in the remainder
            let rem_digits = ((remainder as f32).log10() + 1.0) as u32;
            // use the number of digits in the remainder to decide how many to show in the decimal
            let decimal = (remainder / 10u64.pow(3 - rem_digits.min(3))) as usize;
            format!("{}.{:03}k", k, decimal)
        } else {
            format!("{}k", k)
        }
    } else {
        n.to_string()
    }
}