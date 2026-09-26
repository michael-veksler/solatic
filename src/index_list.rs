use anyhow::{Error, Result};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Index(u32);

const CTRL_BLOCK_IDX: Index = Index(0);

impl Index {
    pub fn is_valid(&self) -> bool {
        *self != CTRL_BLOCK_IDX
    }
    fn offset(&self) -> usize {
        self.0 as usize
    }
}
#[derive(Debug, Clone, Copy, Default)]
struct Node<T> {
    value: T,
    prev: Index,
    next: Index,
}
#[derive(Debug, Clone)]
pub struct IndexList<T> {
    data: Vec<Node<T>>,
}

impl<T: Default> Default for IndexList<T> {
    fn default() -> Self {
        Self {
            data: vec![Node {
                value: T::default(),
                prev: CTRL_BLOCK_IDX,
                next: CTRL_BLOCK_IDX,
            }],
        }
    }
}
impl<T: Default + Clone> IndexList<T> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_empty(&self) -> bool {
        self.data.len() <= 1
    }
    pub fn len(&self) -> u32 {
        (self.data.len() - 1) as u32
    }
    pub fn front_index(&self) -> Index {
        self.data[CTRL_BLOCK_IDX.offset()].next
    }
    pub fn back_index(&self) -> Index {
        self.data[CTRL_BLOCK_IDX.offset()].prev
    }
    fn unlink(&mut self, index: Index) {
        let prev = self.data[index.offset()].prev;
        let next = self.data[index.offset()].next;
        self.data[prev.offset()].next = next;
        self.data[next.offset()].prev = prev;
    }
    fn link(&mut self, left: Index, right: Index) {
        self.data[left.offset()].next = right;
        self.data[right.offset()].prev = left;
    }
    pub fn front(&self) -> Option<&T> {
        let head = self.front_index();
        if head.is_valid() {
            Some(&self.data[head.offset()].value)
        } else {
            None
        }
    }
    pub fn front_mut(&mut self) -> Option<&mut T> {
        let head = self.front_index();
        if head.is_valid() {
            Some(&mut self.data[head.offset()].value)
        } else {
            None
        }
    }

    pub fn get(&self, index: Index) -> Option<&T> {
        if index.is_valid() && index.offset() < self.data.len() {
            Some(&self.data[index.offset()].value)
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        if index.is_valid() && index.offset() < self.data.len() {
            Some(&mut self.data[index.offset()].value)
        } else {
            None
        }
    }
    pub fn push_front(&mut self, value: &T) -> &mut T {
        let new_front = Index((self.data.len()) as u32);
        let old_front = self.front_index();
        self.data.push(Node {
            value: value.clone(),
            prev: CTRL_BLOCK_IDX,
            next: old_front,
        });
        self.data[CTRL_BLOCK_IDX.offset()].next = new_front;
        self.data[old_front.offset()].prev = new_front;
        &mut self.data.last_mut().unwrap().value
    }
    pub fn push_back(&mut self, value: &T) -> &mut T {
        let new_back = Index((self.data.len()) as u32);
        let old_back = self.back_index();
        self.data.push(Node {
            value: value.clone(),
            prev: old_back,
            next: CTRL_BLOCK_IDX,
        });
        self.data[CTRL_BLOCK_IDX.offset()].prev = new_back;
        self.data[old_back.offset()].next = new_back;
        &mut self.data.last_mut().unwrap().value
    }
    pub fn move_to_front(&mut self, index: Index) -> Result<()> {
        if !index.is_valid() || index.offset() >= self.data.len() {
            return Err(Error::msg("move_to_front: Invalid index"));
        }
        self.unlink(index);
        self.link(index, self.front_index());
        self.link(CTRL_BLOCK_IDX, index);
        Ok(())
    }

    /**
     * Returns the next index after the given index in the list.
     *
     * If the given index is invalid or out of bounds, return the first valid index in the list
     */
    pub fn next(&self, index: Index) -> Index {
        let index = if index.offset() < self.data.len() {
            index
        } else {
            CTRL_BLOCK_IDX
        };
        self.data[index.offset()].next
    }

    /**
     * Returns the previous index before the given index in the list.
     *
     * If the given index is invalid or out of bounds, return the last valid index in the list
     */
    pub fn prev(&self, index: Index) -> Index {
        let index = if index.offset() < self.data.len() {
            index
        } else {
            CTRL_BLOCK_IDX
        };
        self.data[index.offset()].prev
    }
}
