use anyhow::{Error, Result};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EntryHandle(u32);

const CTRL_BLOCK_HANDLE: EntryHandle = EntryHandle(0);

impl EntryHandle {
    pub fn is_valid(&self) -> bool {
        *self != CTRL_BLOCK_HANDLE
    }

    // Implementation only offset vs the internal data vector index.
    // For user visible indices use handle.index() instead.
    fn impl_offset(&self) -> usize {
        self.0 as usize
    }
    pub fn index(&self) -> Option<usize> {
        if self.is_valid() {
            Some(self.0 as usize - 1)
        } else {
            None
        }
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

    fn extend_with(&mut self, new_len: usize, element: T) {
        assert!(new_len > self.len(), "new_len must be greater than current length");
        let old_len = self.len();
        self.data.resize(
            new_len + 1,
            Node {
                value: element,
                prev: CTRL_BLOCK_HANDLE,
                next: CTRL_BLOCK_HANDLE,
            },
        );
        let mut prev_back = self.back_handle();
        for i in old_len..new_len {
            let handle = EntryHandle(i as u32 + 1);
            self.link(prev_back, handle);
            prev_back = handle;
        }
        self.link(prev_back, CTRL_BLOCK_HANDLE);
    }

    /**
     * Truncates the list to the specified new length.
     * Elements with indices greater than or equal to new_len are removed from the list.
     */
    pub fn truncate_indices(&mut self, new_len: usize) {
        for i in new_len + 1..=self.len() {
            let handle = EntryHandle(i as u32);
            self.unlink(handle);
        }
        self.data.truncate(new_len + 1);
    }

    /**
     * Resizes the list to the specified new length.
     *
     * If the new length is greater than the current length, the list is extended with the provided element.
     *
     * If the new length is less than the current length, the list is truncated.
     * The effect is as if elem[i] is dropped from the linked list for any i >= new_len.
     */
    pub fn resize_indices(&mut self, new_len: usize, element: T) {
        if new_len > self.len() {
            self.extend_with(new_len, element);
        } else if new_len < self.len() {
            self.truncate_indices(new_len);
        }
    }

    /**
     * Reorders the list according to the given indices.
     * The indices should be in the range of the current list length.
     *
     * The parameter can be a collection of indices or an iterator returning indices,
     * or anything else that can be converted into an iterator of indices.
     */
    pub fn relink_by_indices<U>(&mut self, indexes: impl IntoIterator<Item = U>)
    where
        U: Into<usize>,
    {
        let mut prev = CTRL_BLOCK_HANDLE;
        for index in indexes.into_iter() {
            let current = EntryHandle(index.into() as u32 + 1);
            self.link(prev, current);
            prev = current;
        }
        self.link(prev, CTRL_BLOCK_HANDLE);
    }

    pub fn is_empty(&self) -> bool {
        self.data.len() <= 1
    }
    pub fn len(&self) -> usize {
        self.data.len() - 1
    }
    pub fn front_handle(&self) -> EntryHandle {
        self.data[CTRL_BLOCK_HANDLE.impl_offset()].next
    }
    pub fn back_handle(&self) -> EntryHandle {
        self.data[CTRL_BLOCK_HANDLE.impl_offset()].prev
    }
    fn unlink(&mut self, handle: EntryHandle) {
        let prev = self.data[handle.impl_offset()].prev;
        let next = self.data[handle.impl_offset()].next;
        self.data[prev.impl_offset()].next = next;
        self.data[next.impl_offset()].prev = prev;
    }
    fn link(&mut self, left: EntryHandle, right: EntryHandle) {
        self.data[left.impl_offset()].next = right;
        self.data[right.impl_offset()].prev = left;
    }
    pub fn front(&self) -> Option<&T> {
        let head = self.front_handle();
        if head.is_valid() {
            Some(&self.data[head.impl_offset()].value)
        } else {
            None
        }
    }
    pub fn front_mut(&mut self) -> Option<&mut T> {
        let head = self.front_handle();
        if head.is_valid() {
            Some(&mut self.data[head.impl_offset()].value)
        } else {
            None
        }
    }

    pub fn get(&self, handle: EntryHandle) -> Option<&T> {
        if handle.is_valid() && handle.impl_offset() < self.data.len() {
            Some(&self.data[handle.impl_offset()].value)
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, handle: EntryHandle) -> Option<&mut T> {
        if handle.is_valid() && handle.impl_offset() < self.data.len() {
            Some(&mut self.data[handle.impl_offset()].value)
        } else {
            None
        }
    }
    pub fn push_front(&mut self, value: T) {
        let new_front = EntryHandle((self.data.len()) as u32);
        let old_front = self.front_handle();
        self.data.push(Node {
            value,
            prev: CTRL_BLOCK_HANDLE,
            next: old_front,
        });
        self.data[CTRL_BLOCK_HANDLE.impl_offset()].next = new_front;
        self.data[old_front.impl_offset()].prev = new_front;
    }
    pub fn push_back(&mut self, value: T) {
        let new_back = EntryHandle((self.data.len()) as u32);
        let old_back = self.back_handle();
        self.data.push(Node {
            value,
            prev: old_back,
            next: CTRL_BLOCK_HANDLE,
        });
        self.data[CTRL_BLOCK_HANDLE.impl_offset()].prev = new_back;
        self.data[old_back.impl_offset()].next = new_back;
    }
    pub fn move_to_front(&mut self, handle: EntryHandle) -> Result<()> {
        if !handle.is_valid() || handle.impl_offset() >= self.data.len() {
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
        let handle = if handle.impl_offset() < self.data.len() {
            handle
        } else {
            CTRL_BLOCK_HANDLE
        };
        self.data[handle.impl_offset()].next
    }

    /**
     * Returns the previous handle before the given handle in the list.
     *
     * If the given handle is invalid or out of bounds, return the last valid handle in the list
     */
    pub fn prev(&self, handle: EntryHandle) -> EntryHandle {
        let handle = if handle.impl_offset() < self.data.len() {
            handle
        } else {
            CTRL_BLOCK_HANDLE
        };
        self.data[handle.impl_offset()].prev
    }
}

impl<T: Default + Clone + Copy> FromIterator<T> for InplaceList<T> {
    fn from_iter<I>(into_iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = into_iter.into_iter();
        let mut ret: InplaceList<T> = Self::default();
        if let Some(size) = iter.size_hint().1 {
            ret.data.reserve(size);
        }
        for element in iter {
            ret.push_back(element);
        }
        ret
    }
}

impl<T: Default + Clone + Copy, const N: usize> From<[T; N]> for InplaceList<T> {
    fn from(array: [T; N]) -> Self {
        Self::from_iter(array)
    }
}

impl<T> Index<usize> for InplaceList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index + 1].value
    }
}

impl<T> IndexMut<usize> for InplaceList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index + 1].value
    }
}
