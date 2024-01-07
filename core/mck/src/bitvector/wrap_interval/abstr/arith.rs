use super::Bitvector;
use crate::forward::HwArith;

impl<const L: u32> HwArith for Bitvector<L> {
    fn arith_neg(self) -> Self {
        // arithmetic negation
        // since we use wrapping arithmetic, same as subtracting the value from 0
        HwArith::sub(Self::new(0), self)
    }

    fn add(self, rhs: Self) -> Self {
        // ensure the produced bounds are less than 2^L apart, produce a full interval otherwise
        if self.addsub_becomes_full(rhs) {
            Self::full()
        } else {
            // wrapping and fully monotonic: add bounds
            let start = self.start.add(rhs.start);
            let end = self.end.add(rhs.end);

            Self { start, end }
        }
    }

    fn sub(self, rhs: Self) -> Self {
        // ensure the produced bounds are less than 2^L apart, produce a full interval otherwise
        if self.addsub_becomes_full(rhs) {
            Self::full()
        } else {
            // wrapping, monotonic on lhs, anti-monotonic on rhs: subtract bounds, remember to flip rhs bounds
            let start = self.start.sub(rhs.end);
            let end = self.end.sub(rhs.start);

            Self { start, end }
        }
    }

    fn mul(self, rhs: Self) -> Self {
        todo!()
    }

    fn udiv(self, rhs: Self) -> Self {
        todo!()
    }

    fn sdiv(self, rhs: Self) -> Self {
        todo!()
    }

    fn urem(self, rhs: Self) -> Self {
        todo!()
    }

    fn srem(self, rhs: Self) -> Self {
        todo!()
    }
}

impl<const L: u32> Bitvector<L> {
    fn addsub_becomes_full(self, rhs: Self) -> bool {
        let Some(self_len) = self.len() else {
            return true;
        };
        let Some(rhs_len) = rhs.len() else {
            return true;
        };

        let wrapped_total_len = self_len.add(rhs_len);
        wrapped_total_len.as_unsigned() < self_len.as_unsigned()
            || wrapped_total_len.as_unsigned() < rhs_len.as_unsigned()
    }
}
