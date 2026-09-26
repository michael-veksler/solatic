use anyhow::Result;
use solatic::index_list::{Index, IndexList};
#[test]
fn list_is_empty() {
    let list: IndexList<i32> = IndexList::new();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
    assert!(!list.front_index().is_valid());
    assert!(list.front().is_none());
    assert!(list.get(list.front_index()).is_none());

    let mut mut_list = list;
    assert!(mut_list.front_mut().is_none());
    assert!(mut_list.get_mut(mut_list.front_index()).is_none());
}

#[test]
fn list_is_single() {
    let list: IndexList<i32> = {
        let mut mut_list = IndexList::new();
        mut_list.push_front(&123);
        mut_list
    };
    assert!(!list.is_empty());
    assert_eq!(list.len(), 1);
    assert!(list.front_index().is_valid());
    assert_eq!(list.front(), Some(&123));
    assert_eq!(list.get(list.front_index()), Some(&123));
    assert!(!list.next(list.front_index()).is_valid());

    let mut mut_list = list;
    assert_eq!(mut_list.front_mut(), Some(&mut 123));
    assert_eq!(mut_list.get_mut(mut_list.front_index()), Some(&mut 123));

    *mut_list.front_mut().unwrap() = 345;
    assert_eq!(mut_list.front(), Some(&345));
    assert!(!mut_list.next(mut_list.front_index()).is_valid());
}

#[test]
fn push_front() {
    let mut list = IndexList::new();
    list.push_front(&1);
    list.push_front(&2);
    list.push_front(&3);

    assert!(!list.is_empty());
    assert_eq!(list.len(), 3);
    assert_eq!(list.front(), Some(&3));

    let mut index = list.front_index();
    let mut values = Vec::new();
    while index.is_valid() {
        values.push(*list.get(index).unwrap());
        index = list.next(index);
    }
    assert_eq!(values, &[3, 2, 1]);
}

fn fwd_values<T: Clone + Default>(list: &IndexList<T>) -> Vec<T> {
    let mut index = list.front_index();
    let mut values = Vec::new();
    while index.is_valid() {
        values.push(list.get(index).unwrap().clone());
        index = list.next(index);
    }
    values
}
fn bwd_values<T: Clone + Default>(list: &IndexList<T>) -> Vec<T> {
    let mut index = list.back_index();
    let mut values = Vec::new();
    while index.is_valid() {
        values.push(list.get(index).unwrap().clone());
        index = list.prev(index);
    }
    values
}

#[test]
fn push_back() {
    let mut list = IndexList::new();
    list.push_back(&1);
    list.push_back(&2);
    list.push_back(&3);

    assert_eq!(fwd_values(&list), &[1, 2, 3]);
    assert_eq!(bwd_values(&list), &[3, 2, 1]);
}

fn indices_to_values<T: Clone + Default>(list: &IndexList<T>, indices: &[Index]) -> Vec<Option<T>> {
    indices.iter().map(|&index| list.get(index).cloned()).collect()
}
#[test]
fn push_back_front() {
    let mut list = IndexList::new();
    let mut push_back_indices: Vec<Index> = Vec::new();
    let mut push_front_indices: Vec<Index> = Vec::new();

    list.push_back(&1);
    push_back_indices.push(list.front_index());

    list.push_front(&-1);
    push_front_indices.push(list.front_index());

    list.push_back(&2);
    push_back_indices.push(list.back_index());

    list.push_front(&-2);
    push_front_indices.push(list.front_index());

    assert_eq!(fwd_values(&list), &[-2, -1, 1, 2]);
    assert_eq!(bwd_values(&list), &[2, 1, -1, -2]);
    assert_eq!(indices_to_values(&list, &push_back_indices), &[Some(1), Some(2)]);
    assert_eq!(indices_to_values(&list, &push_front_indices), &[Some(-1), Some(-2)]);
}

#[test]
fn test_invalid_index() {
    let list: IndexList<i32> = IndexList::new();

    assert_eq!(indices_to_values(&list, &[list.back_index()]), &[None]);
}
#[test]
fn move_to_front3() -> Result<()> {
    let mut list = IndexList::new();
    list.push_back(&1);
    list.push_back(&2);
    list.push_back(&3);

    let second_index = list.next(list.front_index());
    assert_eq!(list.get(second_index), Some(&2));

    list.move_to_front(second_index)?;
    assert_eq!(list.front_index(), second_index);
    assert_eq!(fwd_values(&list), &[2, 1, 3]);
    assert_eq!(bwd_values(&list), &[3, 1, 2]);

    list.move_to_front(list.front_index())?;
    assert_eq!(fwd_values(&list), &[2, 1, 3]);
    assert_eq!(bwd_values(&list), &[3, 1, 2]);

    list.move_to_front(list.back_index())?;
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
    let mut list = IndexList::new();
    list.push_back(&1);

    list.move_to_front(list.front_index())?;
    assert_eq!(fwd_values(&list), &[1]);
    assert_eq!(bwd_values(&list), &[1]);
    Ok(())
}

#[test]
fn move_to_front_invalid() -> Result<()> {
    let mut list = IndexList::new();
    list.push_back(&1);
    list.push_back(&2);

    let invalid_index = list.next(list.back_index());
    assert!(list.move_to_front(invalid_index).is_err());

    // make sure the failed move_to_front didn't disrupt the list
    assert_eq!(fwd_values(&list), &[1, 2]);
    assert_eq!(bwd_values(&list), &[2, 1]);

    // Make sure move_to_front still works
    list.move_to_front(list.back_index())?;
    assert_eq!(fwd_values(&list), &[2, 1]);
    assert_eq!(bwd_values(&list), &[1, 2]);
    Ok(())
}

#[test]
fn prev_next_out_of_bounds() {
    let mut list = IndexList::new();
    list.push_back(&1);
    list.push_back(&2);

    let mut long_list = IndexList::new();
    for i in 0..10 {
        long_list.push_back(&i);
    }

    let out_of_bounds_index = long_list.back_index();
    assert_eq!(list.next(out_of_bounds_index), list.front_index());
    assert_eq!(list.prev(out_of_bounds_index), list.back_index());
}
