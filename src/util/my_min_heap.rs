use std::collections::HashMap;
use std::hash::Hash;

enum Justify {
    Left,
    Right,
    Root
}

pub struct MyMinHeap<P: PartialOrd, K: Hash+Eq+Clone, V=()> {
    vec: Vec<Box<(K, P, V)>>,
    table: HashMap<K, usize>
}

impl <P: PartialOrd, K: Hash+Clone+Eq, V> MyMinHeap<P, K, V> {
    pub fn new() -> Self {
        return MyMinHeap {
            vec: vec![],
            table: HashMap::new()
        }
    }
    fn swap(&mut self, i1: usize, k1: K, i2: usize, k2: Option<K>) {
        if i1 != i2 {
            self.table.insert(k1, i2);
            if let Some(k2) = k2 {
                self.table.insert(k2, i1);
            }
            self.vec.swap(i1, i2);
        }
    }
    fn justify_heap(&mut self, index: usize) -> Justify {
        assert!(index < self.vec.len(), "MyMinHeap: 超出陣列範圍");
        let root = &self.vec[index];
        if index * 2 + 1 < self.vec.len() {
            let left = &self.vec[index * 2 + 1];
            if index * 2 + 2 < self.vec.len() {
                let right = &self.vec[index * 2 + 2];
                if right.1 < root.1 && right.1 < left.1 {
                    // 與右手交換
                    self.swap(index, root.0.clone(), index*2+2, Some(right.0.clone()));
                    return Justify::Right;
                }
            }
            if left.1 < root.1 {
                self.swap(index, root.0.clone(), index*2 + 1, Some(left.0.clone()));
                return Justify::Left;
            }
        }
        return Justify::Root;
    }
    pub fn push(&mut self, id: K, priority: P, value: V) {
        assert!(!self.contains_key(&id), "MyMinHeap: 欲加入已存在的鍵");
        let mut index = self.vec.len();
        self.vec.push(Box::new((id.clone(), priority, value)));
        self.table.insert(id, index);
        while index > 0 {
            index = (index - 1) / 2;
            if let Justify::Root = self.justify_heap(index) {
                break;
            }
        }
    }
    pub fn pop(&mut self) -> Option<(K, P, V)> {
        if self.vec.len() == 0 {
            return None;
        }
        let head = &self.vec[0];
        let tail = &self.vec[self.vec.len()-1];
        self.table.remove(&head.0).unwrap();
        self.swap(self.vec.len() - 1, tail.0.clone(), 0, None);
        let ret = self.vec.pop().unwrap();
        let mut index = 0;
        while index < self.vec.len() {
            match self.justify_heap(index) {
                Justify::Root => break,
                Justify::Left => index = index * 2 + 1,
                Justify::Right => index = index * 2,
            }
        }
        return Some(*ret);
    }
    pub fn contains_key(&self, id: &K) -> bool {
        return self.table.contains_key(&id);
    }
    pub fn len(&self) -> usize {
        return self.vec.len();
    }
    pub fn decrease_priority(&mut self, id: &K, new_priority: P) {
        assert!(self.contains_key(id), "MyMinHeap: 欲修改不存在鍵的權重");
        let mut index = *(self.table.get(&id).unwrap());
        let mut entry = &mut self.vec[index];
        if entry.1 > new_priority {
            entry.1 = new_priority;
            loop {
                if index > 0 {
                    index = (index - 1) / 2;
                }
                if let Justify::Root = self.justify_heap(index) {
                    break;
                }
            }
        }
    }
    pub fn peak(&self) -> &(K, P, V) {
        return &self.vec[0];
    }
    pub fn get(&self, id: &K) -> Option<(&P, &V)> {
        if self.contains_key(id) {
            let index = *self.table.get(&id).unwrap();
            return Some((&self.vec[index].1, &self.vec[index].2));
        } else {
            return None;
        }
    }
}


#[cfg(test)]
mod test {
    type TestV = i32;
    use super::MyMinHeap;
    #[test]
    fn test_push_pop_contain() {
        let mut heap: MyMinHeap<f64, i32, TestV> = MyMinHeap::new();
        heap.push(2, 2.0, 20);
        heap.push(3, 1.0, 30);
        heap.push(1, 3.0, 10);

        let contains_3 = heap.contains_key(&3); 
        assert_eq!(true, contains_3);

        assert_eq!(heap.pop().unwrap(), (3, 1.0, 30));
        let contains_3 = heap.contains_key(&3); 
        assert_eq!(false, contains_3);

        let contains_4 = heap.contains_key(&4); 
        assert_eq!(false, contains_4);

        heap.push(4, 0.0, 40);
        heap.push(5, 2.1, 50);

        let contains_4 = heap.contains_key(&4); 
        assert_eq!(true, contains_4);

        assert_eq!(heap.pop().unwrap(), (4, 0.0, 40));
        assert_eq!(heap.pop().unwrap(), (2, 2.0, 20));
        assert_eq!(heap.pop().unwrap(), (5, 2.1, 50));
    }
    #[test]
    fn test_push_pop_peak_decrease() {
        let mut heap: MyMinHeap<f64, i32, TestV> = MyMinHeap::new();
        heap.push(3, 1.0, 30);
        heap.push(1, 3.0, 10);
        heap.push(2, 2.0, 20);

        assert_eq!(*heap.peak(), (3, 1.0, 30));
        heap.decrease_priority(&1, 0.5);
        assert_eq!(heap.pop().unwrap(), (1, 0.5, 10));

        assert_eq!(*heap.peak(), (3, 1.0, 30));
        heap.decrease_priority(&2, 0.5);
        assert_eq!(heap.pop().unwrap(), (2, 0.5, 20));
    }
}