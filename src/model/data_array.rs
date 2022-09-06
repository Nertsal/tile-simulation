pub struct DataArray<T> {
    inner: Vec<T>,
}

impl<T> DataArray<T> {
    pub fn new(size: usize, default_element: T) -> Self
    where
        T: Clone,
    {
        Self {
            inner: vec![default_element; size],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.inner.iter_mut()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.inner.get_mut(index)
    }
}
