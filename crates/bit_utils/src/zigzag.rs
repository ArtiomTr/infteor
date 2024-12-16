pub fn encode(v: i64) -> u64 {
    ((v >> (i64::BITS - 1)) ^ (v << 1)) as u64
}

pub fn decode(v: u64) -> i64 {
    (v >> 1) as i64 ^ -((v & 1) as i64)
}

#[cfg(test)]
mod test {
    use crate::zigzag::decode;

    use super::encode;

    #[test]
    pub fn should_properly_encode_zero() {
        assert_eq!(encode(0), 0);
        assert_eq!(decode(0), 0);
    }

    #[test]
    pub fn should_properly_encode_one() {
        assert_eq!(encode(1), 2);
        assert_eq!(decode(2), 1);
    }

    #[test]
    pub fn should_properly_encode_minus_one() {
        assert_eq!(encode(-1), 1);
        assert_eq!(decode(1), -1);
    }

    #[test]
    pub fn should_properly_encode_minus_two() {
        assert_eq!(encode(-2), 3);
        assert_eq!(decode(3), -2);
    }

    #[test]
    pub fn should_properly_encode_max_i64() {
        assert_eq!(encode(i64::MAX), u64::MAX - 1);
        assert_eq!(decode(u64::MAX - 1), i64::MAX);
    }

    #[test]
    pub fn should_properly_encode_min_i64() {
        assert_eq!(encode(i64::MIN), u64::MAX);
        assert_eq!(decode(u64::MAX), i64::MIN);
    }
}
