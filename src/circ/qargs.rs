use std::{borrow::{Borrow, BorrowMut}, ops::{Deref, DerefMut}};

use derive_more::{Debug, Display};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[debug("{:?}", self.to_vec())]
#[display("{:?}", self.to_vec())]
pub struct QArgs16([u8; 2]);

impl QArgs16 {
    pub fn new() -> Self {
        QArgs16([u8::MAX; 2])
    }
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.iter().cloned().collect()
    }
    pub fn len(&self) -> usize {
        if self.0[0] == u8::MAX {
            0
        } else if self.0[1] == u8::MAX {
            1
        } else {
            2
        }
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.0[..self.len()]
    }
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        let len = self.len();
        &mut self.0[..len]
    }
    pub fn iter(&self) -> impl Iterator<Item = &u8> + Clone {
        self.0.iter().take(self.len())
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut u8>  {
        let len = self.len();
        self.0.iter_mut().take(len)
    }

    pub fn into_iter(self) -> impl Iterator<Item = u8> {
        self.0.into_iter().take(self.len())
    }
    pub fn contains(&self, q: &u8) -> bool {
        self.0[..self.len()].contains(&q)
    }
}

impl FromIterator<u8> for QArgs16 {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let mut qargs = QArgs16::new();
        for (i, q) in iter.into_iter().enumerate() {
            qargs.0[i] = q;
        }
        qargs
    }
}

impl std::ops::Index<usize> for QArgs16 {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(self.0[index] != u8::MAX, "Index out of bounds");
        &self.0[index]
    }
}

impl AsRef<[u8]> for QArgs16 {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl AsMut<[u8]> for QArgs16 {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

impl Borrow<[u8]> for QArgs16 {
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl BorrowMut<[u8]> for QArgs16 {
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

impl Deref for QArgs16 {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl DerefMut for QArgs16 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<const N: usize> From<[u8; N]> for QArgs16 {
    fn from(arr: [u8; N]) -> Self {
        arr.into_iter().collect()
    }
}