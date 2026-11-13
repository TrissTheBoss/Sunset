pub fn floor_div(dividend: i32, divisor: i32) -> i32 {
    if dividend == i32::MIN && divisor == -1 {
        return dividend;
    }
    let mut quotient = dividend / divisor;
    if (dividend ^ divisor) < 0 && quotient.wrapping_mul(divisor) != dividend {
        quotient -= 1;
    }
    quotient
}

pub fn floor_mod(dividend: i32, divisor: i32) -> i32 {
    dividend.wrapping_sub(floor_div(dividend, divisor).wrapping_mul(divisor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_div_matches_java_for_negative_dividends() {
        assert_eq!(floor_div(-1, 16), -1);
        assert_eq!(floor_div(-16, 16), -1);
        assert_eq!(floor_div(-17, 16), -2);
        assert_eq!(floor_div(17, 16), 1);
        assert_eq!(floor_div(16, 16), 1);
        assert_eq!(floor_div(0, 16), 0);
    }

    #[test]
    fn floor_mod_is_non_negative_for_positive_divisor() {
        assert_eq!(floor_mod(-1, 16), 15);
        assert_eq!(floor_mod(-16, 16), 0);
        assert_eq!(floor_mod(-17, 16), 15);
        assert_eq!(floor_mod(17, 16), 1);
        assert_eq!(floor_mod(0, 16), 0);
    }

    #[test]
    fn handles_extreme_integers() {
        assert_eq!(floor_div(i32::MIN, 16), i32::MIN >> 4);
        assert_eq!(floor_mod(i32::MIN, 16), 0);
        assert_eq!(floor_div(i32::MAX, 16), i32::MAX / 16);
        assert_eq!(floor_mod(i32::MAX, 16), 15);
        assert_eq!(floor_div(i32::MIN, -1), i32::MIN);
        assert_eq!(floor_mod(i32::MIN, -1), 0);
    }

    #[test]
    fn property_floor_mod_identity() {
        let mut state = 0x9e3779b97f4a7c15u64;
        for _ in 0..10_000 {
            let a = next_u32(&mut state) as i32;
            let b = (next_u32(&mut state) % 100 + 1) as i32;
            let modulus = floor_mod(a, b);
            assert!(modulus >= 0);
            assert!(modulus < b);
            assert_eq!(modulus, a.wrapping_sub(floor_div(a, b).wrapping_mul(b)));
        }
    }

    #[test]
    fn property_floor_div_rounds_toward_negative_infinity() {
        let mut state = 0x243f6a8885a308d3u64;
        for _ in 0..10_000 {
            let a = next_u32(&mut state) as i32;
            let b = (next_u32(&mut state) % 100 + 1) as i32;
            let q = floor_div(a, b);
            assert!((q as i64) * (b as i64) <= a as i64);
            assert!((q as i64 + 1) * (b as i64) > a as i64);
        }
    }

    fn next_u32(state: &mut u64) -> u32 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        x as u32
    }
}
