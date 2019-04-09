use std::borrow::Borrow;
use std::collections::{hash_map::RandomState, HashMap, HashSet};
use std::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};
use std::marker::PhantomData;

/// A hasher to be used with things that are already hashes
#[derive(Default)]
struct KnownHasher {
    hash: Option<u64>,
}

impl Hasher for KnownHasher {
    #[inline]
    fn write(&mut self, _: &[u8]) {
        panic!("KnownHasher must be called with known u64 hash values")
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        debug_assert!(self.hash.is_none());
        self.hash = Some(i);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash.expect("Nothing was hashed") as u64
    }
}

/// `Trash` is a hash, and can be used directly with `TrashMap`
/// and `TrashSet` to interact with entries
///
/// Think of it as an identifier for a map or set entry
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Trash(u64);

impl Trash {
    pub fn get_hash(&self) -> u64 {
        self.0
    }
}

/// A hash map that can operate on known hash values (`Trash`) instead of actual keys
///
/// Sometimes you need to access the same element in the hashmap multiple times and
/// don't wish to re-hash each time. In such a case, you can use `TrashMap`, which
/// will provide a `Trash` id that can be used to cheaply access map values as long
/// as you keep it around.
///
/// An assumption made here is that there are no hash collisions in the `u64` hash space
/// for your hasher. If there are, this may result in unpredictable behavior.
///
///
/// ```
/// use trashmap::TrashMap;
/// # fn do_stuff(x: &mut TrashMap<str, &'static str>) {}
///
/// let mut map = TrashMap::new();
/// let id = map.insert("foo", "bar");
/// do_stuff(&mut map);
/// assert!(map.get(id) == Some(&"bar"));
/// map.remove(id);
/// ```
pub struct TrashMap<K: ?Sized, V, S = RandomState> {
    hasher: S,
    map: HashMap<Trash, V, BuildHasherDefault<KnownHasher>>,
    key: PhantomData<*const K>,
}

impl<K: ?Sized, V, S> TrashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    /// Construct a basic TrashMap
    pub fn new() -> Self
    where
        S: Default,
    {
        Self {
            hasher: Default::default(),
            map: Default::default(),
            key: PhantomData,
        }
    }

    /// Construct a basic trashmap with a custom hasher and/or capacity
    pub fn with_capacity_and_hasher(cap: usize, hasher: S) -> Self {
        Self {
            hasher,
            map: HashMap::with_capacity_and_hasher(cap, Default::default()),
            key: PhantomData,
        }
    }

    /// Inserts a key-value pair, returning the `Trash` id for the entry
    pub fn insert<Q: ?Sized>(&mut self, k: &Q, v: V) -> Trash
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let trash = self.trash(k);
        self.map.insert(trash, v);
        trash
    }

    /// Inserts a key-value pair, using the `Trash` id for the key
    pub fn insert_id(&mut self, k: Trash, v: V)
    {
        self.map.insert(k, v);
    }

    /// Inserts a key-value pair, returning the `Trash` id for the entry as well
    /// as the old entry, if present
    pub fn insert_replace<Q: ?Sized>(&mut self, k: &Q, v: V) -> (Trash, Option<V>)
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let trash = self.trash(k);
        (trash, self.map.insert(trash, v))
    }

    /// Gets the entry corresponding to a given `Trash` id, if present
    pub fn get(&self, key: Trash) -> Option<&V> {
        self.map.get(&key)
    }

    /// Gets the entry corresponding to a given key, if present.
    /// Also returns the `Trash` id of the key to avoid recomputation
    pub fn get_key<Q: ?Sized>(&self, key: &Q) -> (Option<&V>, Trash)
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let trash = self.trash(key);
        (self.map.get(&trash), trash)
    }

    /// Removes and returns the entry corresponding to a given `Trash` id,
    /// if present
    pub fn remove(&mut self, key: Trash) -> Option<V> {
        self.map.remove(&key)
    }

    pub fn remove_key<Q: ?Sized>(&mut self, key: &Q) -> (Option<V>, Trash)
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let trash = self.trash(key);
        (self.map.remove(&trash), trash)
    }

    fn trash<Q: ?Sized>(&self, k: &Q) -> Trash
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut state = self.hasher.build_hasher();
        k.hash(&mut state);
        Trash(state.finish())
    }
}

impl<K, V, S> Default for TrashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    fn default() -> Self {
        Self::new()
    }
}


/// A hash set that can operate on known hash values (`Trash`) instead of actual keys
///
/// Sometimes you need to access the same element in the set multiple times and
/// don't wish to re-hash each time. In such a case, you can use `TrashMap`, which
/// will provide a `Trash` id that can be used to cheaply access map values as long
/// as you keep it around.
///
/// An assumption made here is that there are no hash collisions in the `u64` hash space
/// for your hasher. If there are, this may result in unpredictable behavior.
///
///
/// ```
/// use trashmap::TrashSet;
/// # fn do_stuff(x: &mut TrashSet<str>) {}
///
/// let mut map = TrashSet::new();
/// let id = map.insert("foo");
/// do_stuff(&mut map);
/// assert!(map.contains(id));
/// map.remove(id);
/// ```
///
///
/// For example, this is useful if you're doing some kind of recursion-prevention
/// scheme:
///
/// ```rust
/// use trashmap::TrashSet;
/// # fn lookup_children(entry: &str) -> &'static [&'static str] { &[] }
/// struct State {
///    seen: TrashSet<str>,
/// }
///
/// impl State {
///     fn step_into(&mut self, entry: &str) {
///         let (contains, id) = self.seen.contains_key(entry);
///         if contains {
///             panic!("found recursive loop!");
///         }
///         self.seen.insert_id(id);
///         let children = lookup_children(entry);
///         for child in children {
///            self.step_into(child);
///         }
///         self.seen.remove(id);  
///     }  
/// }
/// ```
pub struct TrashSet<K: ?Sized, S = RandomState> {
    hasher: S,
    set: HashSet<Trash, BuildHasherDefault<KnownHasher>>,
    key: PhantomData<*const K>,
}

impl<K: ?Sized, S> TrashSet<K, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    pub fn new() -> Self
    where
        S: Default,
    {
        Self {
            hasher: Default::default(),
            set: Default::default(),
            key: PhantomData,
        }
    }

    pub fn with_capacity_and_hasher(cap: usize, hasher: S) -> Self {
        Self {
            hasher,
            set: HashSet::with_capacity_and_hasher(cap, Default::default()),
            key: PhantomData,
        }
    }

    /// Insert a key, getting a `Trash` id to be used to access the entry later
    pub fn insert<Q: ?Sized>(&mut self, key: &Q) -> Trash
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let trash = self.trash(key);
        self.set.insert(trash);
        trash
    }

    /// Insert an element based on its `Trash` id
    pub fn insert_id(&mut self, key: Trash)
    {
        self.set.insert(key);
    }

    /// Check if the `Trash` id has been inserted before
    pub fn contains(&self, key: Trash) -> bool {
        self.set.contains(&key)
    }

    /// Check if the key has been inserted before
    ///
    /// Also returns the `Trash` id for the key
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> (bool, Trash)
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let trash = self.trash(key);
        (self.set.contains(&trash), trash)
    }

    /// Remove an entry based on its `Trash` id
    pub fn remove(&mut self, key: Trash) -> bool {
        self.set.remove(&key)
    }

    /// Remove a key
    pub fn remove_key<Q: ?Sized>(&mut self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.set.remove(&self.trash(key))
    }

    fn trash<Q: ?Sized>(&self, k: &Q) -> Trash
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut state = self.hasher.build_hasher();
        k.hash(&mut state);
        Trash(state.finish())
    }
}

impl<K, S> Default for TrashSet<K, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    fn default() -> Self {
        Self::new()
    }
}
