use std::{cmp::Ordering, vec};


const SORTING_NETWORKS : [&[(u8, u8)]; 9] = [
    &[],
    &[],
    &[(0,1)],
    &[(0,2), (0,1), (1,2)],
    &[(0,2),(1,3), (0,1),(2,3), (1,2)],
    &[(0,3),(1,4), (0,2),(1,3), (0,1),(2,4), (1,2),(3,4), (2,3)],
    &[(0,5),(1,3),(2,4), (1,2),(3,4), (0,3),(2,5), (0,1),(2,3),(4,5), (1,2),(3,4)],
    &[(0,6),(2,3),(4,5), (0,2),(1,4),(3,6), (0,1),(2,5),(3,4), (1,2),(4,6), (2,3),(4,5), (1,2),(3,4),(5,6)],
    &[(0,2),(1,3),(4,6),(5,7), (0,4),(1,5),(2,6),(3,7), (0,1),(2,3),(4,5),(6,7), (2,4),(3,5), (1,4),(3,6), (1,2),(3,4),(5,6)]
];

#[derive(derive_more::Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[debug("{self}")]
pub struct Permut32 {
    pub raw: u32
}

impl Permut32 {
    pub fn new() -> Self {
        Permut32 { raw: 0 }
    }
    pub fn identity(len: u8) -> Self {
        assert!(len <= 8, "Permutation length must be less than or equal to 8");
        let mut result = Permut32::new();
        for i in 0..len {
            result.set(i, i);
        }
        result.set_len(len);
        return result;
    }
    pub fn from_iter(perm: impl Iterator<Item=u8>) -> Self {
        let mut len = 0;
        let mut result = Permut32::new();
        let mut mask = 0;
        for p in perm {
            assert!(p < 8, "Permutation values must be less than 8");
            assert!((mask >> p) & 1 == 0, "Permutation values must be unique");
            mask |= 1 << p;
            result.set(len, p);
            len += 1;
        }
        result.set_len(len);
        return result;
    }
    pub fn from_iter_unchecked(perm: impl Iterator<Item=u8>) -> Self {
        let mut len = 0;
        let mut result = Permut32::new();
        for p in perm {
            result.set(len, p);
            len += 1;
        }
        result.set_len(len);
        return result;
    }
    pub fn from_iter_with_ext(len: u8, mut perm: impl Iterator<Item=u8>) -> Self {
        let mut result = Permut32::new();
        result.set_len(len);
        
        let mut mask = 0u8;

        for i in 0..len {
            let a = if let Some(p) = perm.next() {
                p
            } else if mask & (1 << i) == 0 {
                i
            } else {
                mask.trailing_ones() as u8
            };
            assert!((mask >> a) & 1 == 0, "Permutation values must be unique");
            mask |= 1 << a;
            result.set(i, a);
        }
        return result;
    }
    pub fn from_order(len: u8, mut compare: impl FnMut(u8, u8) -> Ordering) -> Self {
        assert!(len <= 8, "Permutation length must be less than or equal to 8");
        let mut values: Vec<u8> = (0..len).collect();
        values.sort_by(|&a, &b| compare(a, b));
        // let mut result = Permut32::new();
        // for (i, &value) in values.iter().enumerate() {
        //     result.set(value, i as u8);
        // }
        // result.set_len(values.len() as u8);
        // result
        return Permut32::from_iter_unchecked(values.into_iter());
    }
    pub fn inv(&self) -> Self {
        let mut inv = Permut32::new();
        for i in 0..self.len() {
            inv.set(self.at(i), i);
        }
        inv.set_len(self.len());
        inv
    }
    pub fn len(&self) -> u8 {
        ((self.raw >> 3 * 8) & 7) as u8
    }
    pub fn at(&self, idx: u8) -> u8 {
        assert!(idx < 8);
        ((self.raw >> 3 * idx) & 7) as u8
    }
    pub fn apply(self, perm: impl Iterator<Item=u8>) -> impl Iterator<Item=u8> {
        perm.map(move |x| {
            let idx = x as u8;
            assert!(idx < self.len(), "Index out of bounds for permutation");
            self.at(idx)
        })
    }
    pub fn apply_vec(&self, perm: &[u8]) -> Vec<u8> {
        self.apply(perm.iter().cloned()).collect()
    }
    fn set(&mut self, idx: u8, value: u8) {
        self.raw = self.raw & !(7 << (3 * idx)) | ((value as u32) << (3 * idx));
    }
    fn set_len(&mut self, value: u8) {
        self.set(8, value);
    }
    pub fn iter(&self) -> Permut32Iter {
        Permut32Iter {
            perm: *self,
            idx: 0,
        }
    }
    pub fn swap_inputs(&mut self, a: u8, b: u8) {
        assert!(a < 8 && b < 8, "Indices must be less than 8");
        let a_val = self.at(a);
        let b_val = self.at(b);
        self.set(a, b_val);
        self.set(b, a_val);
    }
    pub fn permut_bv(&self, mut bv: u8) -> u8 {
        let mut result = 0u8;
        for i in 0..self.len() {
            if bv & 1 != 0 {
                result |= 1 << self.at(i);
            }
            bv >>= 1;
        }
        result
    }
    pub fn generate_swaps(&self) -> impl Iterator<Item=(u8,u8)> + 'static {
        let mut step = self.clone();
        SORTING_NETWORKS[self.len() as usize].iter().cloned().filter(move |(a, b)| {
            if step.at(*a) > step.at(*b) {
                step.swap_inputs(*a, *b);
                true
            } else { false }
        })
    }
    pub fn is_identity(&self) -> bool {
        for i in 0..self.len() {
            if self.at(i) != i {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Permut32Iter{
    pub perm: Permut32,
    pub idx: u8,
}

impl Iterator for Permut32Iter {
    type Item=u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.perm.len() {
            let value = self.perm.at(self.idx);
            self.idx += 1;
            Some(value)
        } else {
            None
        }
    }
    fn count(self) -> usize {
        self.perm.len() as usize - self.idx as usize
    }
}

impl std::ops::Mul for Permut32 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        assert!(self.len() == other.len());
        Permut32::from_iter_unchecked((0..other.len()).map(|x| {
            self.at(other.at(x as u8))
        }))
    }
}

impl std::fmt::Display for Permut32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        let mut it = self.iter();
        if let Some(first) = it.next() {
            write!(f, "{first}")?;
        }
        for a in it {
            write!(f, " {a}")?;
        }
        write!(f, ")")
    }
}



#[cfg(test)]
mod test {
    use crate::groups::permutation::Permut32;

    #[test]
    fn test() {
        let perm = Permut32::from_iter([0, 2, 1].into_iter());
        println!("{}", perm);
        println!("{}", perm.permut_bv(0b101));
    }
}