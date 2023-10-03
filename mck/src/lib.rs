mod bitvector;
pub mod mark;
mod traits;
mod util;

pub use bitvector::concr::MachineBitvector;
pub use bitvector::mark::MarkBitvector;
pub use bitvector::three_valued::ThreeValuedBitvector;

pub use traits::AbstractInput;
pub use traits::AbstractMachine;
pub use traits::AbstractState;
pub use traits::FabricatedIterator;
pub use traits::Fabricator;
pub use traits::FieldManipulate;
pub use traits::MachineDiv;
pub use traits::MachineExt;
pub use traits::MachineShift;
pub use traits::TypedCmp;
pub use traits::TypedEq;

pub use mark::MarkInput;
pub use mark::MarkMachine;
pub use mark::MarkState;
