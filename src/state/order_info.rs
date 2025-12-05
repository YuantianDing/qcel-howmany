use std::sync::LazyLock;

use pyo3::pyclass;

use crate::{groups::permutation::Permut32, utils::FmtJoinIter};

#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyclass(eq, str)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderInfo {
    pub permut: Vec<usize>,
    pub eq_classes: Vec<usize>,
}

impl std::fmt::Display for OrderInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OrderInfo[{}]", self.eqclasses().map(|x| format!("{}", x.iter().fjoin("="))).fjoin(" "))
    }
}

impl OrderInfo {
    pub fn get_eqclass<'a>(&'a self, idx: usize) -> &'a [usize] {
        let start = if idx > 0 { self.eq_classes[idx - 1] } else { 0 };
        let end = self.eq_classes[idx];

        &self.permut[start..end]
    }
    pub fn eqclasses<'a>(&'a self) -> impl Iterator<Item = &'a [usize]> + Clone {
        (0..self.n_eqclasses()).map(move |i| self.get_eqclass(i))
    }
    pub fn get_eqclass_mut<'a>(&'a mut self, idx: usize) -> &'a mut [usize] {
        let start = if idx > 0 { self.eq_classes[idx - 1] } else { 0 };
        let end = self.eq_classes[idx];

        &mut self.permut[start..end]
    }

    pub fn get_eqclass_range(&self, idx: usize) -> std::ops::Range<usize> {
        let start = if idx > 0 { self.eq_classes[idx - 1] } else { 0 };
        let end = self.eq_classes[idx];

        start..end
    }

    pub fn sort_eqclass_by_key<T: Ord>(&mut self, mut idx: usize, keyf: impl Fn(&usize) -> T) {
        let range = self.get_eqclass_range(idx);
        self.permut[range.clone()].sort_by_key(&keyf);
        for i in range.start + 1..range.end {
            if keyf(&self.permut[i]) != keyf(&self.permut[i - 1]) {
                self.eq_classes.insert(idx, i);
                idx += 1;
            }
        }
    }
    pub fn sort_eqclass_by_array<T: Ord + std::fmt::Debug>(&mut self, mut idx: usize, keyf: impl IntoIterator<Item = T>) -> bool {
        let mut vec: Vec<_> = keyf.into_iter().zip(self.get_eqclass(idx).iter().cloned()).collect();
        assert!(vec.len() == self.get_eqclass(idx).len());
        vec.sort();
        let range = self.get_eqclass_range(idx);
        let mut flag = false;
        for i in range.start + 1..range.end {
            if vec[i - range.start].0 != vec[i - 1 - range.start].0 {
                self.eq_classes.insert(idx, i);
                idx += 1;
                flag = true;
            }
        }
        for (loc, v) in self.permut[range].iter_mut().zip(vec.into_iter().map(|(_, v)| v)) {
            *loc = v;
        }
        flag
    }
    pub fn as_perms(&self) -> impl Iterator<Item=Permut32> + Clone {
        let (perm, eqmask) = self.as_bits();
        EQMASK_TO_PERMS[eqmask as usize >> 1].iter().cloned().map(move |p| perm * p.shrink(perm.len()))
    }
    pub fn as_perms_mask(&self, mask: u8) -> impl Iterator<Item=Permut32> + Clone {
        let (perm, eqmask) = self.as_bits();
        EQMASK_TO_PERMS[(eqmask & perm.inv().permut_bv(mask)) as usize >> 1].iter().cloned().map(move |p| perm * p.shrink(perm.len()))
    }
}

#[pyo3_stub_gen::derive::gen_stub_pymethods]
#[pyo3::pymethods]
impl OrderInfo {
    #[new]
    pub fn new(size: usize) -> Self {
        Self {
            permut: (0..size).collect(),
            eq_classes: vec![size],
        }
    }

    pub fn n_eqclasses(&self) -> usize {
        self.eq_classes.len()
    }
    pub fn has_eq(&self) -> bool {
        self.n_eqclasses() < self.permut.len()
    }
    pub fn first_eqclass(&self) -> Option<usize> {
        (0..self.n_eqclasses()).find(|idx| self.get_eqclass(*idx).len() > 1)
    }
    pub fn first_eqclass_after(&self, idx: usize) -> Option<usize> {
        (idx..self.n_eqclasses()).find(|idx| self.get_eqclass(*idx).len() > 1)
    }

    pub fn as_bits(&self) -> (Permut32, u8) {
        if self.permut.len() < 2 {
            return (Permut32::identity(self.permut.len() as u8), 0)
        }
        let mut eqmask = (1u8 << self.permut.len()) - 2;
        for a in self.eq_classes.iter() {
            eqmask &= !(1u8 << *a as u8);
        }
        (Permut32::from_iter_unchecked(self.permut.iter().map(|a| *a as u8)), eqmask)
    }
    
}

static EQMASK_TO_PERMS: LazyLock<Vec<Vec<Permut32>>> = LazyLock::new(|| (0..1 << 4).map(|eqmask| {
    let eqmask = eqmask << 1;
    let mut eqsets = Vec::new();
    for i in 0..5 {
        if (eqmask & (1 << i)) == 0 {
            eqsets.push(vec![i as u8]);
        } else {
            eqsets.last_mut().unwrap().push(i as u8);
        }
    }
    
    Permut32::all(5).iter().cloned().filter(|p| {
        eqsets.iter().all(|eqset| eqset.iter().all(|i| eqset.contains(&p.at(*i))))
    }).collect()
}).collect());

#[cfg(test)]
mod test {
    use crate::state::order_info::EQMASK_TO_PERMS;

    #[test]
    fn test1() {
        let mut oi = super::OrderInfo::new(10);
        oi.sort_eqclass_by_key(0, |x| x % 3);
        assert_eq!(oi.n_eqclasses(), 3);
        assert_eq!(oi.get_eqclass(0), &[0, 3, 6, 9]);
        assert_eq!(oi.get_eqclass(1), &[1, 4, 7]);
        assert_eq!(oi.get_eqclass(2), &[2, 5, 8]);

        oi.sort_eqclass_by_key(1, |x| std::cmp::Reverse(*x));
        assert_eq!(oi.n_eqclasses(), 5);
    }
    #[test]
    fn test2() {
        let mut oi = super::OrderInfo::new(10);
        oi.sort_eqclass_by_array(0, [0, 1, 2, 0, 1, 2, 0, 1, 2, 0]);
        assert_eq!(oi.n_eqclasses(), 3);
        assert_eq!(oi.get_eqclass(0), &[0, 3, 6, 9]);
        assert_eq!(oi.get_eqclass(1), &[1, 4, 7]);
        assert_eq!(oi.get_eqclass(2), &[2, 5, 8]);

        oi.sort_eqclass_by_array(1, [2, 1, 0]);
        assert_eq!(oi.n_eqclasses(), 5);
    }
    #[test]
    fn test3() {
        println!("{:?}", EQMASK_TO_PERMS);
    }
}