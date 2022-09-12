#[derive(Clone)]
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

    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.inner.get_mut(index)
    }

    /// Attempts to get immutable access to two elemets.
    /// Returns `None` if any of the elemets is not present.
    pub fn get_two(&self, index: usize, other: usize) -> Option<(&T, &T)> {
        self.inner
            .get(index)
            .and_then(|a| self.inner.get(other).map(|b| (a, b)))
    }
}

impl<T> IntoIterator for DataArray<T> {
    type Item = T;

    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<T> From<Vec<T>> for DataArray<T> {
    fn from(vec: Vec<T>) -> Self {
        Self { inner: vec }
    }
}
