mod bitvector;
pub mod mark;
mod traits;
mod util;

pub use bitvector::concr::MachineBitvector;
pub use bitvector::mark::MarkBitvector;
pub use bitvector::three_valued::ThreeValuedBitvector;

pub use traits::mark::MarkInput;
pub use traits::mark::MarkMachine;
pub use traits::mark::MarkState;
pub use traits::*;
