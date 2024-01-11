use super::Interval;

impl Interval {
    pub fn bit_and(self, rhs: Self) -> Self {
        Interval::new(self.bit_and_min(rhs), self.bit_and_max(rhs))
    }

    pub fn bit_or(self, rhs: Self) -> Self {
        Interval::new(self.bit_or_min(rhs), self.bit_or_max(rhs))
    }

    pub fn bit_xor(self, rhs: Self) -> Self {
        Interval::new(self.bit_xor_min(rhs), self.bit_xor_max(rhs))
    }

    fn bit_and_min(self, rhs: Self) -> u64 {
        // from Hacker's Delight, "Propagating Bounds through Logical Operations", with speedup
        let (mut a, b, mut c, d) = (self.min, self.max, rhs.min, rhs.max);
        let mut m = if !(a ^ c) == 0 {
            0
        } else {
            1u64 << (u64::BITS - 1 - (!(a ^ c)).leading_zeros())
        };
        while m != 0 {
            if (!a & !c & m) != 0 {
                let temp = (a | m) & (0u64.wrapping_sub(m));
                if temp <= b {
                    a = temp;
                    break;
                }
                let temp = (c | m) & (0u64.wrapping_sub(m));
                if temp <= d {
                    c = temp;
                    break;
                }
            }
            m >>= 1;
        }
        a & c
    }

    fn bit_and_max(self, rhs: Self) -> u64 {
        // from Hacker's Delight, "Propagating Bounds through Logical Operations", with speedup
        let (a, mut b, c, mut d) = (self.min, self.max, rhs.min, rhs.max);
        let mut m = if (b | d) == 0 {
            0
        } else {
            1u64 << (u64::BITS - 1 - (b | d).leading_zeros())
        };
        while m != 0 {
            if (b & !d & m) != 0 {
                let temp = (b & !m) | (m - 1);
                if temp >= a {
                    b = temp;
                    break;
                }
            } else if (!b & d & m) != 0 {
                let temp = (d & !m) | (m - 1);
                if temp >= c {
                    d = temp;
                    break;
                }
            }
            m >>= 1;
        }
        b & d
    }

    fn bit_or_min(self, rhs: Self) -> u64 {
        // from Hacker's Delight, "Propagating Bounds through Logical Operations", with speedup
        let (mut a, b, mut c, d) = (self.min, self.max, rhs.min, rhs.max);
        let mut m = if (a ^ c) == 0 {
            0
        } else {
            1u64 << (u64::BITS - 1 - (a ^ c).leading_zeros())
        };
        while m != 0 {
            if (!a & c & m) != 0 {
                let temp = (a | m) & (0u64.wrapping_sub(m));
                if temp <= b {
                    a = temp;
                    break;
                }
            } else if (a & !c & m) != 0 {
                let temp = (c | m) & (0u64.wrapping_sub(m));
                if temp <= d {
                    c = temp;
                    break;
                }
            }
            m >>= 1;
        }
        a | c
    }

    fn bit_or_max(self, rhs: Self) -> u64 {
        // from Hacker's Delight, "Propagating Bounds through Logical Operations", with speedup
        let (a, mut b, c, mut d) = (self.min, self.max, rhs.min, rhs.max);
        let mut m = if (b & d) == 0 {
            0
        } else {
            1u64 << (u64::BITS - 1 - (b & d).leading_zeros())
        };
        while m != 0 {
            if (b & d & m) != 0 {
                let temp = (b - m) | (m - 1);
                if temp >= a {
                    b = temp;
                    break;
                }
                let temp = (d - m) | (m - 1);
                if temp >= c {
                    d = temp;
                    break;
                }
            }
            m >>= 1;
        }
        b | d
    }

    fn bit_xor_min(self, rhs: Self) -> u64 {
        // from Hacker's Delight, "Propagating Bounds through Logical Operations", with speedup
        let (mut a, b, mut c, d) = (self.min, self.max, rhs.min, rhs.max);
        let mut m = if (a ^ c) == 0 {
            0
        } else {
            1u64 << (u64::BITS - 1 - (a ^ c).leading_zeros())
        };
        while m != 0 {
            if (!a & c & m) != 0 {
                let temp = (a | m) & (0u64.wrapping_sub(m));
                if temp <= b {
                    a = temp;
                }
            } else if (a & !c & m) != 0 {
                let temp = (c | m) & (0u64.wrapping_sub(m));
                if temp <= d {
                    c = temp;
                }
            }
            m >>= 1;
        }
        a ^ c
    }

    fn bit_xor_max(self, rhs: Self) -> u64 {
        // from Hacker's Delight, "Propagating Bounds through Logical Operations", with speedup
        let (a, mut b, c, mut d) = (self.min, self.max, rhs.min, rhs.max);
        let mut m = if (b & d) == 0 {
            0
        } else {
            1u64 << (u64::BITS - 1 - (b & d).leading_zeros())
        };
        while m != 0 {
            if (b & d & m) != 0 {
                let mut temp = (b - m) | (m - 1);
                if temp >= a {
                    b = temp;
                } else {
                    temp = (d - m) | (m - 1);
                    if temp >= c {
                        d = temp;
                    }
                }
            }
            m >>= 1;
        }
        b ^ d
    }
}
