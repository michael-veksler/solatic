use anyhow::Result;
use solatic::inplace_list::{EntryHandle, InplaceList};
#[test]
fn list_is_empty() {
    let list: InplaceList<i32> = InplaceList::new();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
    assert!(!list.front_handle().is_valid());
    assert!(list.front().is_none());
    assert!(list.get(list.front_handle()).is_none());

    let mut mut_list = list;
    assert!(mut_list.front_mut().is_none());
    assert!(mut_list.get_mut(mut_list.front_handle()).is_none());
}

#[test]
fn list_is_single() {
    let list: InplaceList<i32> = {
        let mut mut_list = InplaceList::new();
        mut_list.push_front(123);
        mut_list
    };
    assert!(!list.is_empty());
    assert_eq!(list.len(), 1);
    assert!(list.front_handle().is_valid());
    assert_eq!(list.front(), Some(&123));
    assert_eq!(list.get(list.front_handle()), Some(&123));
    assert!(!list.next(list.front_handle()).is_valid());

    let mut mut_list = list;
    assert_eq!(mut_list.front_mut(), Some(&mut 123));
    assert_eq!(mut_list.get_mut(mut_list.front_handle()), Some(&mut 123));

    *mut_list.front_mut().unwrap() = 345;
    assert_eq!(mut_list.front(), Some(&345));
    assert!(!mut_list.next(mut_list.front_handle()).is_valid());
}

#[test]
fn push_front() {
    let mut list = InplaceList::new();
    list.push_front(1);
    list.push_front(2);
    list.push_front(3);

    assert!(!list.is_empty());
    assert_eq!(list.len(), 3);
    assert_eq!(list.front(), Some(&3));

    let mut handle = list.front_handle();
    let mut values = Vec::new();
    while handle.is_valid() {
        values.push(*list.get(handle).unwrap());
        handle = list.next(handle);
    }
    assert_eq!(values, &[3, 2, 1]);
}

fn fwd_values<T: Clone + Default + Eq + std::fmt::Debug>(list: &InplaceList<T>) -> Vec<T> {
    let mut handle = list.front_handle();
    let mut values = Vec::new();
    while handle.is_valid() {
        let value = list.get(handle).unwrap().clone();
        assert_eq!(list[handle.index().unwrap()], value);
        assert_eq!(list.get_handle(handle.index().unwrap()), handle);
        values.push(value);
        handle = list.next(handle);
    }
    assert_eq!(handle.index(), None);
    assert_eq!(list.get_handle(list.len()), handle);
    values
}
fn bwd_values<T: Clone + Default>(list: &InplaceList<T>) -> Vec<T> {
    let mut handle = list.back_handle();
    let mut values = Vec::new();
    while handle.is_valid() {
        values.push(list.get(handle).unwrap().clone());
        handle = list.prev(handle);
    }
    values
}

#[test]
fn push_back() {
    let mut list = InplaceList::default(); // same as empty()
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);

    assert_eq!(fwd_values(&list), &[1, 2, 3]);
    assert_eq!(bwd_values(&list), &[3, 2, 1]);
}

fn handles_to_values<T: Clone + Default>(list: &InplaceList<T>, handles: &[EntryHandle]) -> Vec<Option<T>> {
    handles.iter().map(|&handle| list.get(handle).cloned()).collect()
}
#[test]
fn push_back_front() {
    let mut list = InplaceList::new(); // same as default()
    let mut push_back_handles: Vec<EntryHandle> = Vec::new();
    let mut push_front_handles: Vec<EntryHandle> = Vec::new();

    list.push_back(1);
    push_back_handles.push(list.front_handle());

    list.push_front(-1);
    push_front_handles.push(list.front_handle());

    list.push_back(2);
    push_back_handles.push(list.back_handle());

    list.push_front(-2);
    push_front_handles.push(list.front_handle());

    assert_eq!(fwd_values(&list), &[-2, -1, 1, 2]);
    assert_eq!(bwd_values(&list), &[2, 1, -1, -2]);
    assert_eq!(handles_to_values(&list, &push_back_handles), &[Some(1), Some(2)]);
    assert_eq!(handles_to_values(&list, &push_front_handles), &[Some(-1), Some(-2)]);
}

#[test]
fn test_invalid_handle() {
    let list: InplaceList<i32> = InplaceList::new();

    assert_eq!(handles_to_values(&list, &[list.back_handle()]), &[None]);
}
#[test]
fn move_to_front3() -> Result<()> {
    let mut list = InplaceList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);

    let second_handle = list.next(list.front_handle());
    assert_eq!(list.get(second_handle), Some(&2));

    list.move_to_front(second_handle)?;
    assert_eq!(list.front_handle(), second_handle);
    assert_eq!(fwd_values(&list), &[2, 1, 3]);
    assert_eq!(bwd_values(&list), &[3, 1, 2]);

    list.move_to_front(list.front_handle())?;
    assert_eq!(fwd_values(&list), &[2, 1, 3]);
    assert_eq!(bwd_values(&list), &[3, 1, 2]);

    list.move_to_front(list.back_handle())?;
    assert_eq!(fwd_values(&list), &[3, 2, 1]);
    assert_eq!(bwd_values(&list), &[1, 2, 3]);

    list.push_back(0);
    list.push_front(4);
    assert_eq!(fwd_values(&list), &[4, 3, 2, 1, 0]);
    assert_eq!(bwd_values(&list), &[0, 1, 2, 3, 4]);
    Ok(())
}

#[test]
fn move_to_front1() -> Result<()> {
    let mut list = InplaceList::new();
    list.push_back(1);

    list.move_to_front(list.front_handle())?;
    assert_eq!(fwd_values(&list), &[1]);
    assert_eq!(bwd_values(&list), &[1]);
    Ok(())
}

#[test]
fn move_to_front_invalid() -> Result<()> {
    let mut list = InplaceList::from([1, 2]);

    let invalid_handle = list.next(list.back_handle());
    assert!(list.move_to_front(invalid_handle).is_err());

    // make sure the failed move_to_front didn't disrupt the list
    assert_eq!(fwd_values(&list), &[1, 2]);
    assert_eq!(bwd_values(&list), &[2, 1]);

    // Make sure move_to_front still works
    list.move_to_front(list.back_handle())?;
    assert_eq!(fwd_values(&list), &[2, 1]);
    assert_eq!(bwd_values(&list), &[1, 2]);
    Ok(())
}

#[test]
fn prev_next_out_of_bounds() {
    let list = InplaceList::from([1, 2]);
    let long_list = InplaceList::from_iter(0..10);

    let out_of_bounds_handle = long_list.back_handle();
    assert_eq!(list.next(out_of_bounds_handle), list.front_handle());
    assert_eq!(list.prev(out_of_bounds_handle), list.back_handle());
}

#[test]
fn index_and_index_mut() {
    let mut list = InplaceList::from([1, 2, 3, -1]);

    assert_eq!(list[0], 1);
    assert_eq!(list[1], 2);
    assert_eq!(list[2], 3);
    assert_eq!(list[3], -1);

    list[0] = 10;
    list[1] = 20;
    list[2] = 30;
    list[3] = -10;

    assert_eq!(list[0], 10);
    assert_eq!(list[1], 20);
    assert_eq!(list[2], 30);
    assert_eq!(list[3], -10);

    assert_eq!(fwd_values(&list), &[10, 20, 30, -10]);
    assert_eq!(bwd_values(&list), &[-10, 30, 20, 10]);
}

#[test]
fn relink_by_indices() {
    let mut list = InplaceList::from([6, 10, 3, 1, 5, 7, 9, 2, 4, 8]);
    let order_indices: [usize; 10] = [3, 7, 2, 8, 4, 0, 5, 9, 6, 1];
    list.relink_by_indices(order_indices);
    assert_eq!(fwd_values(&list), (1..=10).collect::<Vec<u32>>());
    assert_eq!(bwd_values(&list), (1..=10).rev().collect::<Vec<u32>>());

    list.relink_by_indices(order_indices.into_iter().rev());
    assert_eq!(fwd_values(&list), (1..=10).rev().collect::<Vec<u32>>());
    assert_eq!(bwd_values(&list), (1..=10).collect::<Vec<u32>>());
}

#[test]
fn resize() -> Result<()> {
    let mut list: InplaceList<i32> = InplaceList::from_iter(0..4);
    list.resize_indices(8, -1);

    assert_eq!(fwd_values(&list), &[0, 1, 2, 3, -1, -1, -1, -1]);
    assert_eq!(bwd_values(&list), &[-1, -1, -1, -1, 3, 2, 1, 0]);
    for i in 4..8 {
        list[i] = i as i32;
    }
    assert_eq!(fwd_values(&list), &[0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(bwd_values(&list), &[7, 6, 5, 4, 3, 2, 1, 0]);
    list.move_to_front(list.back_handle())?;
    assert_eq!(fwd_values(&list), &[7, 0, 1, 2, 3, 4, 5, 6]);
    assert_eq!(bwd_values(&list), &[6, 5, 4, 3, 2, 1, 0, 7]);
    list.resize_indices(10, -2);
    assert_eq!(fwd_values(&list), &[7, 0, 1, 2, 3, 4, 5, 6, -2, -2]);
    assert_eq!(bwd_values(&list), &[-2, -2, 6, 5, 4, 3, 2, 1, 0, 7]);
    for i in 8..10 {
        list[i] = i as i32;
    }
    list.move_to_front(list.back_handle())?;
    assert_eq!(fwd_values(&list), &[9, 7, 0, 1, 2, 3, 4, 5, 6, 8]);
    assert_eq!(bwd_values(&list), &[8, 6, 5, 4, 3, 2, 1, 0, 7, 9]);

    list.resize_indices(4, -1);
    assert_eq!(fwd_values(&list), &[0, 1, 2, 3]); // Elements with index >= 4 have been removed
    assert_eq!(bwd_values(&list), &[3, 2, 1, 0]); // Elements with index >= 4 have been removed
    list.truncate_indices(2);
    assert_eq!(fwd_values(&list), &[0, 1]); // Elements with index >= 2 have been removed
    assert_eq!(bwd_values(&list), &[1, 0]); // Elements with index >= 2 have been removed
    Ok(())
}

#[test]
fn move_before() {
    let mut list: InplaceList<i32> = InplaceList::from_iter(0..8);

    let invalid = list.prev(list.front_handle());
    list.move_before(list.get_handle(2), invalid);
    assert_eq!(fwd_values(&list), &[0, 1, 3, 4, 5, 6, 7, 2]);
    assert_eq!(bwd_values(&list), &[2, 7, 6, 5, 4, 3, 1, 0]);

    list.move_before(list.back_handle(),invalid);
    assert_eq!(fwd_values(&list), &[0, 1, 3, 4, 5, 6, 7, 2]);
    assert_eq!(bwd_values(&list), &[2, 7, 6, 5, 4, 3, 1, 0]);

    list.move_before(list.front_handle(),invalid);
    assert_eq!(fwd_values(&list), &[1, 3, 4, 5, 6, 7, 2, 0]);
    assert_eq!(bwd_values(&list), &[0, 2, 7, 6, 5, 4, 3, 1]);

    list.move_before(list.get_handle(4), list.back_handle());
    assert_eq!(fwd_values(&list), &[1, 3, 5, 6, 7, 2, 4, 0]);
    assert_eq!(bwd_values(&list), &[0, 4, 2, 7, 6, 5, 3, 1]);

    list.move_before(list.get_handle(3), list.front_handle());
    assert_eq!(fwd_values(&list), &[3, 1, 5, 6, 7, 2, 4, 0]);
    assert_eq!(bwd_values(&list), &[0, 4, 2, 7, 6, 5, 1, 3]);

    list.move_before(invalid, list.front_handle());
    assert_eq!(fwd_values(&list), &[3, 1, 5, 6, 7, 2, 4, 0]);
    assert_eq!(bwd_values(&list), &[0, 4, 2, 7, 6, 5, 1, 3]);

    list.move_before(invalid, invalid);
    assert_eq!(fwd_values(&list), &[3, 1, 5, 6, 7, 2, 4, 0]);
    assert_eq!(bwd_values(&list), &[0, 4, 2, 7, 6, 5, 1, 3]);

    list.move_before(list.get_handle(1), list.get_handle(1));
    assert_eq!(fwd_values(&list), &[3, 1, 5, 6, 7, 2, 4, 0]);
    assert_eq!(bwd_values(&list), &[0, 4, 2, 7, 6, 5, 1, 3]);

    list.move_before(list.get_handle(1), list.next(list.get_handle(1)));
    assert_eq!(fwd_values(&list), &[3, 1, 5, 6, 7, 2, 4, 0]);
    assert_eq!(bwd_values(&list), &[0, 4, 2, 7, 6, 5, 1, 3]);

}
