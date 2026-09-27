use anyhow::Result;
use solatic::index_list::{EntryHandle, InplaceList};
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
        mut_list.push_front(&123);
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
    list.push_front(&1);
    list.push_front(&2);
    list.push_front(&3);

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

fn fwd_values<T: Clone + Default>(list: &InplaceList<T>) -> Vec<T> {
    let mut handle = list.front_handle();
    let mut values = Vec::new();
    while handle.is_valid() {
        values.push(list.get(handle).unwrap().clone());
        handle = list.next(handle);
    }
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
    let mut list = InplaceList::new();
    list.push_back(&1);
    list.push_back(&2);
    list.push_back(&3);

    assert_eq!(fwd_values(&list), &[1, 2, 3]);
    assert_eq!(bwd_values(&list), &[3, 2, 1]);
}

fn indices_to_values<T: Clone + Default>(list: &InplaceList<T>, indices: &[EntryHandle]) -> Vec<Option<T>> {
    indices.iter().map(|&handle| list.get(handle).cloned()).collect()
}
#[test]
fn push_back_front() {
    let mut list = InplaceList::new();
    let mut push_back_indices: Vec<EntryHandle> = Vec::new();
    let mut push_front_indices: Vec<EntryHandle> = Vec::new();

    list.push_back(&1);
    push_back_indices.push(list.front_handle());

    list.push_front(&-1);
    push_front_indices.push(list.front_handle());

    list.push_back(&2);
    push_back_indices.push(list.back_handle());

    list.push_front(&-2);
    push_front_indices.push(list.front_handle());

    assert_eq!(fwd_values(&list), &[-2, -1, 1, 2]);
    assert_eq!(bwd_values(&list), &[2, 1, -1, -2]);
    assert_eq!(indices_to_values(&list, &push_back_indices), &[Some(1), Some(2)]);
    assert_eq!(indices_to_values(&list, &push_front_indices), &[Some(-1), Some(-2)]);
}

#[test]
fn test_invalid_handle() {
    let list: InplaceList<i32> = InplaceList::new();

    assert_eq!(indices_to_values(&list, &[list.back_handle()]), &[None]);
}
#[test]
fn move_to_front3() -> Result<()> {
    let mut list = InplaceList::new();
    list.push_back(&1);
    list.push_back(&2);
    list.push_back(&3);

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

    list.push_back(&0);
    list.push_front(&4);
    assert_eq!(fwd_values(&list), &[4, 3, 2, 1, 0]);
    assert_eq!(bwd_values(&list), &[0, 1, 2, 3, 4]);
    Ok(())
}

#[test]
fn move_to_front1() -> Result<()> {
    let mut list = InplaceList::new();
    list.push_back(&1);

    list.move_to_front(list.front_handle())?;
    assert_eq!(fwd_values(&list), &[1]);
    assert_eq!(bwd_values(&list), &[1]);
    Ok(())
}

#[test]
fn move_to_front_invalid() -> Result<()> {
    let mut list = InplaceList::new();
    list.push_back(&1);
    list.push_back(&2);

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
    let mut list = InplaceList::new();
    list.push_back(&1);
    list.push_back(&2);

    let mut long_list = InplaceList::new();
    for i in 0..10 {
        long_list.push_back(&i);
    }

    let out_of_bounds_handle = long_list.back_handle();
    assert_eq!(list.next(out_of_bounds_handle), list.front_handle());
    assert_eq!(list.prev(out_of_bounds_handle), list.back_handle());
}
