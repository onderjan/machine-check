pub trait FieldManipulate<T> {
    fn get(&self, name: &str) -> Option<T>;
    fn get_mut(&mut self, name: &str) -> Option<&mut T>;
}
