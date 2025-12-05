use std::cmp::Ordering;
use alias_ptr::AliasPtr;
use extension_traits::extension;
use std::hash::{Hash, Hasher};
use std::fmt::{self, Debug, Display};

pub struct AliasCons<T> {
    pub head: T,
    pub tail: AliasList<T>,
}
pub enum AliasList<T> {
    Cons(AliasPtr<AliasCons<T>>),
    Nil,
}

impl<T: 'static> AliasList<T> {
    pub fn nil() -> Self {
        Self::Nil
    }
    pub fn cons(&self, head: T) -> Self {
        Self::Cons(AliasPtr::new(AliasCons { head, tail: self.clone() }))
    }

    pub unsafe fn delete(&mut self) {
        match self {
            Self::Cons(ptr) => ptr.delete(),
            Self::Nil => (),
        }
    }
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'static T> + Clone + 'a {
        self.clone()   
    }
}

impl<T: PartialEq + 'static> PartialEq for AliasList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T: Eq + 'static> Eq for AliasList<T> {}

impl<T: Hash + 'static> Hash for AliasList<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for item in self.iter() {
            item.hash(state);
        }
    }
}

impl<T: Debug + 'static> Debug for AliasList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.iter().collect::<Vec<_>>())
    }
}

impl<T: Display + 'static> Display for AliasList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.iter().map(|item| item.to_string()).collect::<Vec<_>>().join(", "))
    }
}

impl<T> Default for AliasList<T> {
    fn default() -> Self {
        Self::Nil
    }
}

impl<T: Ord + 'static> Ord for AliasList<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T: PartialOrd + 'static> PartialOrd for AliasList<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T> Clone for AliasList<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Cons(ptr) => Self::Cons(ptr.clone()),
            Self::Nil => Self::Nil,
        }
    }
}

impl<T: 'static> FromIterator<T> for AliasList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::Nil;
        for item in iter {
            list = list.cons(item);
        }
        list
    }
}



impl<T: 'static> Iterator for AliasList<T> {
    type Item = &'static T;

    fn next(&mut self) -> Option<Self::Item> {
        match &self {
            AliasList::Cons(ptr) => {
                let item = unsafe { AliasPtr::as_ptr(&ptr).as_ref::<'static>() }.unwrap();
                *self = ptr.tail.clone();
                Some(&item.head)
            }
            AliasList::Nil => None,
        }
    }
}

#[extension(pub trait JoinOptionIter)]
impl<T, Iter: Iterator<Item=T> + Clone> Iter where Self : Sized {
    fn join_option<Sep, Start, End>(&self, sep: Sep, start: Start, end: End) -> JoinOption<Self, Sep, Start, End> {
        JoinOption {
            iter: &self,
            sep,
            start,
            end,
        }
    }
}

#[derive(Clone)]
pub struct JoinOption<'a, Iter, Sep, Start, End> {
    iter: &'a Iter,
    sep: Sep,
    start: Start,
    end: End,
}

impl<'a, T: Debug, Iter: Iterator<Item=T> + Clone, Sep: Debug, Start: Debug, End: Debug> Debug for JoinOption<'a, Iter, Sep, Start, End> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.iter.clone();
        if let Some(a) = iter.next() {
            write!(f, "{:?}{:?}", self.start, a)?;
            for item in iter {
                write!(f, "{:?}{:?}", self.sep, item)?;
            }
            write!(f, "{:?}", self.end)
        } else {
            write!(f, "")
        }
    }
}

impl<'a, T: Display, Iter: Iterator<Item=T> + Clone, Sep: Display, Start: Display, End: Display> Display for JoinOption<'a, Iter, Sep, Start, End> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.iter.clone();
        if let Some(a) = iter.next() {
            write!(f, "{}{}", self.start, a)?;
            for item in iter {
                write!(f, "{}{}", self.sep, item)?;
            }
            write!(f, "{}", self.end)
        } else {
            write!(f, "")
        }
    }
}

pub struct DenseIndexMap {
    pub perm: Vec<Option<usize>>,
    pub count: usize,
}

impl DenseIndexMap {
    pub fn new() -> Self {
        Self {
            perm: Vec::new(),
            count: 0,
        }
    }

    pub fn get_or_insert(&mut self, index: usize) -> usize {
        self.perm.resize(index + 1, None);
        
        if let Some(id) = self.perm[index] {
            return id;
        } else {
            self.perm[index] = Some(self.count);
            self.count += 1;
            return self.count - 1;
        }
    }
    pub fn get(&self, index: usize) -> Option<usize> {
        if index < self.perm.len() {
            self.perm[index]
        } else {
            None
        }
    }
}

pub const fn parse_usize(s: &str) -> usize {
    let mut out: usize = 0;
    let mut i: usize = 0;
    while i<s.len() {
        out *= 10;
        out += (s.as_bytes()[i] - b'0') as usize;
        i += 1;
    }
    out
}