use std::cell::RefCell;
use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};
use alias_ptr::AliasPtr;
use derive_more::Display;
use extension_traits::extension;
use nohash_hasher::BuildNoHashHasher;
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

#[extension(pub trait FmtJoinIter)]
impl<T, Iter: Iterator<Item=T> + Clone> Iter where Self : Sized {
    fn fjoin<Sep>(self, sep: Sep) -> FmtJoin<Self, Sep> {
        FmtJoin {
            iter: RefCell::new(self),
            sep,
            start: FmtNone,
            end: FmtNone,
            orelse: FmtNone,
        }
    }
    fn fjoin_or_else<Sep, OrElse>(self, sep: Sep, orelse: OrElse) -> FmtJoin<Self, Sep, FmtNone, FmtNone, OrElse> {
        FmtJoin {
            iter: RefCell::new(self),
            sep,
            start: FmtNone,
            end: FmtNone,
            orelse,
        }
    }
    fn fjoin_opt_braces<Sep, Start, End>(self, sep: Sep, start: Start, end: End) -> FmtJoin<Self, Sep, Start, End, FmtNone> {
        FmtJoin {
            iter: RefCell::new(self),
            sep,
            start,
            end,
            orelse: FmtNone,
        }
    }
    fn fjoin4<Sep, Start, End, OrElse>(self, sep: Sep, start: Start, end: End, orelse: OrElse) -> FmtJoin<Self, Sep, Start, End, OrElse> {
        FmtJoin {
            iter: RefCell::new(self),
            sep,
            start,
            end,
            orelse,
        }
    }
}

pub struct FmtNone;

impl Debug for FmtNone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Display for FmtNone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
pub struct FmtJoin<Iter, Sep, Start=FmtNone, End=FmtNone, OrElse=FmtNone> {
    iter: RefCell<Iter>,
    sep: Sep,
    start: Start,
    end: End,
    orelse: OrElse,
}

impl<T: Debug, Iter: Iterator<Item=T>, Sep: Debug, Start: Debug, End: Debug, OrElse: Debug> Debug for FmtJoin<Iter, Sep, Start, End, OrElse> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut guard = self.iter.borrow_mut();
        let iter = guard.deref_mut();
        if let Some(a) = iter.next() {
            write!(f, "{:?}{:?}", self.start, a)?;
            for item in iter {
                write!(f, "{:?}{:?}", self.sep, item)?;
            }
            write!(f, "{:?}", self.end)
        } else {
            write!(f, "{:?}", self.orelse)
        }
    }
}

impl<T: Display, Iter: Iterator<Item=T>, Sep: Display, Start: Display, End: Display, OrElse: Debug> Display for FmtJoin<Iter, Sep, Start, End, OrElse> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut guard = self.iter.borrow_mut();
        let iter = guard.deref_mut();
        if let Some(a) = iter.next() {
            write!(f, "{}{}", self.start, a)?;
            for item in iter {
                write!(f, "{}{}", self.sep, item)?;
            }
            write!(f, "{}", self.end)
        } else {
            write!(f, "{:?}", self.orelse)
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

#[derive(Debug, Display, Clone, serde::Deserialize, serde::Serialize)]
pub struct HashTable64<K: Hash + PartialEq, V>(std::collections::HashMap<u64, (K, V), BuildNoHashHasher<u64>>);

impl<K: Hash + PartialEq, V: PartialEq> PartialEq for HashTable64<K, V> {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (k, v) in self.iter() {
            match other.get(k) {
                Some(v2) if v == v2 => (),
                _ => return false,
            }
        }
        true
    }
}

impl<K: Hash + Eq, V: Eq> Eq for HashTable64<K, V> {}

impl<K: Hash + PartialEq, V> HashTable64<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> (u64, Option<V>) {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        match self.0.entry(hash) {
            std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                assert!(occupied_entry.get().0 == key, "Hash collision detected in HashTable64");
                (hash, Some(occupied_entry.insert((key, value)).1))
            },
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert((key, value));
                (hash, None)
            }
        }
    }
    pub fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}
impl<K: Hash + PartialEq, V> HashTable64<K, V> {
    pub fn new() -> Self {
        Self(std::collections::HashMap::with_hasher(BuildNoHashHasher::default()))
    }



    pub fn get(&self, key: &K) -> Option<&V> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        self.0.get(&hash).and_then(|(k, v)| {
            assert!(k == key, "Hash collision detected in HashTable64");
            Some(v)
        })
    }

    pub fn get_mut(&mut self, key: &K) -> Option<(&K, &mut V)> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        self.0.get_mut(&hash).and_then(|(k, v)| {
            assert!(k == key, "Hash collision detected in HashTable64");
            Some((&*k, v))
        })
    }

    pub fn address(&self, hash: u64) -> Option<&(K, V)> {
        self.0.get(&hash)
    }

    pub fn address_mut(&mut self, hash: u64) -> Option<(&K, &mut V)> {
        self.0.get_mut(&hash).map(|(k, v)| (&*k, v))
    }

    pub fn address_or(&mut self, hash: u64, default: (K, V)) -> Option<(&K, &mut V)> {
        match self.0.entry(hash) {
            std::collections::hash_map::Entry::Occupied(occupied_entry) => {
                let (k, v) = occupied_entry.into_mut();
                Some((&*k, v))
            },
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                let (k, v) = vacant_entry.insert(default);
                Some((&*k, v))
            }
        }
    }


    pub fn contains_key(&self, key: &K) -> bool {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        self.0.get(&hash).map_or(false, |(k, _)| k == key)
    }
    pub fn contains_hash(&self, key: u64) -> bool {
        self.0.contains_key(&key)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.0.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.0.values_mut().map(|(_, v)| v)
    }

    pub fn get_or(&mut self, key: K, default: V) -> &mut V {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        match self.0.entry(hash) {
            std::collections::hash_map::Entry::Occupied(occupied_entry) => {
                let (k, v) = occupied_entry.into_mut();
                assert!(k == &key, "Hash collision detected in HashTable64");
                v
            },
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                &mut vacant_entry.insert((key, default)).1
            }
        }
    }

    pub fn remove_hash(&mut self, hash: u64) -> Option<(K, V)> {
        self.0.remove(&hash)
    }
}

impl<K: Hash + Eq, V> IntoIterator for HashTable64<K, V> {
    type Item = (K, V);
    type IntoIter = std::collections::hash_map::IntoValues<u64, (K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_values()
    }
}
// impl<K: Hash + Eq, V> FromIterator<(K, V)> for HashTable64<K, V> {
//     fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
//         let mut table = Self::new();
//         for (k, v) in iter {
//             table.insert(k, v);
//         }
//         table
//     }
// }

impl<K: Hash + Eq, V> Default for HashTable64<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn postcard_write_file<T: serde::Serialize + ?Sized>(t: &T, path: impl AsRef<std::path::Path>) -> postcard::Result<()>
where
    T: serde::Serialize,
{
    
    let file = std::fs::File::create(path).map_err(|_| postcard::Error::SerdeSerCustom)?;
    postcard::to_io(t, file).map(|_| ())
}

pub fn postcard_read_file<T: serde::de::DeserializeOwned>(path: impl AsRef<std::path::Path>) -> postcard::Result<T> {
    let file = std::fs::File::open(path).map_err(|_| postcard::Error::SerdeDeCustom)?;
    postcard::from_io((file, &mut [0u8; 4096][..])).map(|a| a.0)
}