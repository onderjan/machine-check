pub struct FabricatedIterator<T: Fabricator>(T, Option<T::Fabricated>);

impl<T: Fabricator> Iterator for FabricatedIterator<T> {
    type Item = T::Fabricated;

    fn next(&mut self) -> Option<T::Fabricated> {
        let Some(current) = &mut self.1 else {
            return None;
        };
        let result = current.clone();
        let could_increment = Fabricator::increment_fabricated(&self.0, current);
        if !could_increment {
            self.1 = None;
        }
        Some(result)
    }
}

pub trait Fabricator: Sized {
    type Fabricated: Clone;
    fn fabricate_first(&self) -> Self::Fabricated;
    fn increment_fabricated(&self, possibility: &mut Self::Fabricated) -> bool;

    fn into_fabricated_iter(self) -> FabricatedIterator<Self> {
        let first_fabricated = Some(<Self as Fabricator>::fabricate_first(&self));
        FabricatedIterator(self, first_fabricated)
    }
}
