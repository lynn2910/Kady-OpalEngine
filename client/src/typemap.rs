use std::any::TypeId;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait Type: 'static + Send + Sync + Clone {
    type Value;
}

pub type TypeBox = Box<dyn core::any::Any + Send + Sync>;

pub struct TypeMap {
    inner: HashMap<TypeId, TypeBox>
}

impl TypeMap {
    /// Create a new container
    pub fn new() -> Self {
        Self {
            inner: HashMap::new()
        }
    }

    /// Returns number of key & value pairs inside.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns number of key & value pairs inside.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Returns whether a element is present in the map
    pub fn has<T: Type>(&self) -> bool {
        self.inner.contains_key(&TypeId::of::<T>())
    }

    /// Access a element in the map, and return a reference to it, if present
    pub fn get<T: Type>(&self) -> Option<&T> {
        match self.inner.get(&TypeId::of::<T>()) {
            Some(ptr) => match ptr.downcast_ref() {
                Some(res) => Some(res),
                None => unreachable!()
            },
            None => None
        }
    }

    /// Access a element in the map, and return a mutable reference to it, if present
    pub fn get_mut<T: Type>(&mut self) -> Option<&mut T> {
        match self.inner.get_mut(&TypeId::of::<T>()) {
            Some(ptr) => match ptr.downcast_mut() {
                Some(res) => Some(res),
                None => unreachable!()
            },
            None => None
        }
    }

    /// Insert an element inside the map, returning the heap-allocated old one if any
    ///
    /// ## Note
    ///
    /// Be careful when inserting without explicitly specifying type.
    /// Some special types like function pointers are impossible to infer as non-anonymous type.
    /// You should manually specify type when in doubt.
    pub fn insert<T: Type>(&mut self, value: T) -> Option<Box<T>> {
        match self.inner.entry(TypeId::of::<T>()) {
            Entry::Occupied(mut occupied) => {
                let result = occupied.insert(Box::new(value));
                match result.downcast() {
                    Ok(result) => Some(result),
                    Err(_) => unreachable!()
                }
            },
            Entry::Vacant(vacant) => {
                vacant.insert(Box::new(value));
                None
            }
        }
    }

    ///Attempts to remove element from the map, returning boxed `Some` if it is present.
    pub fn remove<T: Type>(&mut self) -> Option<Box<T>> {
        self.inner.remove(&TypeId::of::<T>()).map(|ptr| {
            match ptr.downcast() {
                Ok(result) => result,
                Err(_) => unreachable!()
            }
        })
    }
    
    pub fn get_every_datas(&self) -> Vec<&TypeBox> {
        self.inner.values().collect::<Vec<&TypeBox>>()
    }
}


impl Default for TypeMap {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for TypeMap {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        writeln!(f, "TypeMap {{ size={}, capacity={} }}", self.len(), self.capacity())
    }
}