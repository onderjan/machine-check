pub trait Ext<const X: u32> {
    type Output;

    #[must_use]
    fn ext(self) -> Self::Output;
}
