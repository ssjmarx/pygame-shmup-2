/// Generic object pool for efficient memory management
pub struct ObjectPool<T> {
    objects: Vec<T>,
    active: Vec<bool>,
    free_list: Vec<usize>,
    capacity: usize,
}

impl<T: Clone> ObjectPool<T> {
    pub fn new(capacity: usize, default_value: T) -> Self {
        ObjectPool {
            objects: vec![default_value; capacity],
            active: vec![false; capacity],
            free_list: (0..capacity).collect(),
            capacity,
        }
    }
    
    pub fn allocate(&mut self, value: T) -> Option<usize> {
        if let Some(index) = self.free_list.pop() {
            self.objects[index] = value;
            self.active[index] = true;
            Some(index)
        } else {
            None // Pool full
        }
    }
    
    pub fn deallocate(&mut self, index: usize) {
        if index < self.capacity && self.active[index] {
            self.active[index] = false;
            self.free_list.push(index);
        }
    }
    
    pub fn iter_active(&self) -> impl Iterator<Item = (usize, &T)> {
        self.objects.iter()
            .enumerate()
            .filter(|(i, _)| self.active[*i])
            .map(|(i, obj)| (i, obj))
    }
    
    pub fn iter_active_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.objects.iter_mut()
            .enumerate()
            .filter(|(i, _)| self.active[*i])
            .map(|(i, obj)| (i, obj))
    }
    
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.capacity && self.active[index] {
            Some(&self.objects[index])
        } else {
            None
        }
    }
    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.capacity && self.active[index] {
            Some(&mut self.objects[index])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_object_pool_allocation() {
        let mut pool: ObjectPool<i32> = ObjectPool::new(10, 0);
        let idx = pool.allocate(42);
        assert!(idx.is_some());
        assert_eq!(*pool.get(idx.unwrap()).unwrap(), 42);
    }
    
    #[test]
    fn test_object_pool_deallocation() {
        let mut pool: ObjectPool<i32> = ObjectPool::new(10, 0);
        let idx = pool.allocate(42).unwrap();
        pool.deallocate(idx);
        let idx2 = pool.allocate(99).unwrap();
        assert_eq!(idx, idx2); // Reused same slot
    }
    
    #[test]
    fn test_iter_active_only_returns_active() {
        let mut pool: ObjectPool<i32> = ObjectPool::new(10, 0);
        pool.allocate(1).unwrap();  // Allocated at some index
        let idx2 = pool.allocate(2).unwrap();  // Allocated at another index
        pool.deallocate(idx2);  // Deallocate the one we tracked
        
        let active_count = pool.iter_active().count();
        assert_eq!(active_count, 1);
    }
}