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

    /// Attempts to get mutable access to two elemets.
    /// Returns `None` if any of the elemets is not present of if the indices are equal.
    pub fn get_two_mut(&mut self, index: usize, other: usize) -> Option<(&mut T, &mut T)> {
        let (lower, higher) = match index.cmp(&other) {
            std::cmp::Ordering::Less => (index, other),
            std::cmp::Ordering::Equal => return None,
            std::cmp::Ordering::Greater => (other, index),
        };
        if higher >= self.inner.len() {
            return None;
        }
        let (left, right) = self.inner.as_mut_slice().split_at_mut(higher);
        let lower = &mut left[lower];
        let higher = right.first_mut().unwrap();
        Some((lower, higher))
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
