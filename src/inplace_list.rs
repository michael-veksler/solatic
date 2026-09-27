use anyhow::{Error, Result};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EntryHandle(u32);

const CTRL_BLOCK_HANDLE: EntryHandle = EntryHandle(0);

impl EntryHandle {
    pub fn is_valid(&self) -> bool {
        *self != CTRL_BLOCK_HANDLE
    }
    fn offset(&self) -> usize {
        self.0 as usize
    }
}
#[derive(Debug, Clone, Copy, Default)]
struct Node<T> {
    value: T,
    prev: EntryHandle,
    next: EntryHandle,
}
#[derive(Debug, Clone)]
pub struct InplaceList<T> {
    data: Vec<Node<T>>,
}

impl<T: Default> Default for InplaceList<T> {
    fn default() -> Self {
        Self {
            data: vec![Node {
                value: T::default(),
                prev: CTRL_BLOCK_HANDLE,
                next: CTRL_BLOCK_HANDLE,
            }],
        }
    }
}
impl<T: Default + Clone> InplaceList<T> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_empty(&self) -> bool {
        self.data.len() <= 1
    }
    pub fn len(&self) -> u32 {
        (self.data.len() - 1) as u32
    }
    pub fn front_handle(&self) -> EntryHandle {
        self.data[CTRL_BLOCK_HANDLE.offset()].next
    }
    pub fn back_handle(&self) -> EntryHandle {
        self.data[CTRL_BLOCK_HANDLE.offset()].prev
    }
    fn unlink(&mut self, handle: EntryHandle) {
        let prev = self.data[handle.offset()].prev;
        let next = self.data[handle.offset()].next;
        self.data[prev.offset()].next = next;
        self.data[next.offset()].prev = prev;
    }
    fn link(&mut self, left: EntryHandle, right: EntryHandle) {
        self.data[left.offset()].next = right;
        self.data[right.offset()].prev = left;
    }
    pub fn front(&self) -> Option<&T> {
        let head = self.front_handle();
        if head.is_valid() {
            Some(&self.data[head.offset()].value)
        } else {
            None
        }
    }
    pub fn front_mut(&mut self) -> Option<&mut T> {
        let head = self.front_handle();
        if head.is_valid() {
            Some(&mut self.data[head.offset()].value)
        } else {
            None
        }
    }

    pub fn get(&self, handle: EntryHandle) -> Option<&T> {
        if handle.is_valid() && handle.offset() < self.data.len() {
            Some(&self.data[handle.offset()].value)
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, handle: EntryHandle) -> Option<&mut T> {
        if handle.is_valid() && handle.offset() < self.data.len() {
            Some(&mut self.data[handle.offset()].value)
        } else {
            None
        }
    }
    pub fn push_front(&mut self, value: &T) -> &mut T {
        let new_front = EntryHandle((self.data.len()) as u32);
        let old_front = self.front_handle();
        self.data.push(Node {
            value: value.clone(),
            prev: CTRL_BLOCK_HANDLE,
            next: old_front,
        });
        self.data[CTRL_BLOCK_HANDLE.offset()].next = new_front;
        self.data[old_front.offset()].prev = new_front;
        &mut self.data.last_mut().unwrap().value
    }
    pub fn push_back(&mut self, value: &T) -> &mut T {
        let new_back = EntryHandle((self.data.len()) as u32);
        let old_back = self.back_handle();
        self.data.push(Node {
            value: value.clone(),
            prev: old_back,
            next: CTRL_BLOCK_HANDLE,
        });
        self.data[CTRL_BLOCK_HANDLE.offset()].prev = new_back;
        self.data[old_back.offset()].next = new_back;
        &mut self.data.last_mut().unwrap().value
    }
    pub fn move_to_front(&mut self, handle: EntryHandle) -> Result<()> {
        if !handle.is_valid() || handle.offset() >= self.data.len() {
            return Err(Error::msg("move_to_front: Invalid handle"));
        }
        self.unlink(handle);
        self.link(handle, self.front_handle());
        self.link(CTRL_BLOCK_HANDLE, handle);
        Ok(())
    }

    /**
     * Returns the next handle after the given handle in the list.
     *
     * If the given handle is invalid or out of bounds, return the first valid handle in the list
     */
    pub fn next(&self, handle: EntryHandle) -> EntryHandle {
        let handle = if handle.offset() < self.data.len() {
            handle
        } else {
            CTRL_BLOCK_HANDLE
        };
        self.data[handle.offset()].next
    }

    /**
     * Returns the previous handle before the given handle in the list.
     *
     * If the given handle is invalid or out of bounds, return the last valid handle in the list
     */
    pub fn prev(&self, handle: EntryHandle) -> EntryHandle {
        let handle = if handle.offset() < self.data.len() {
            handle
        } else {
            CTRL_BLOCK_HANDLE
        };
        self.data[handle.offset()].prev
    }
}
