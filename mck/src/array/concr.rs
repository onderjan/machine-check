use crate::bitvector::concr::MachineBitvector;

#[derive(Debug, Clone, Copy)]
pub struct MachineArray<const L: u32, const N: usize, const I: u32> {
    v: [MachineBitvector<L>; N],
}

impl<const E: u32, const N: usize, const I: u32> MachineArray<E, N, I> {
    pub fn filled(fill_element: MachineBitvector<E>) -> Self {
        MachineArray {
            v: [fill_element; N],
        }
    }

    pub fn read(&self, index: MachineBitvector<I>) -> MachineBitvector<E> {
        self.v[index.concrete_unsigned().0 as usize]
    }

    pub fn write(
        &self,
        index: MachineBitvector<I>,
        element: MachineBitvector<E>,
    ) -> MachineArray<E, N, I> {
        let mut result = self.v;
        result[index.concrete_unsigned().0 as usize] = element;
        MachineArray { v: result }
    }
}
