use crate::utils::find_exactly_one;

#[test]
fn test_find_exactly_one() {
    // One item matches the predicate
    assert_eq!(find_exactly_one(vec![1, 2, 3], |n| *n == 2), Some(2));

    // Multiple items match the predicate
    assert_eq!(find_exactly_one(vec![1, 2, 2, 3], |n| *n == 2), None);

    // No items match the predicate
    assert_eq!(find_exactly_one(vec![1, 3, 5], |n| *n == 2), None);

    // Iterable is empty
    assert_eq!(find_exactly_one::<Vec<i32>, _>(vec![], |n| *n == 2), None);

    // Exactly one item matches the predicate in a larger collection
    assert_eq!(
        find_exactly_one(vec![1, 3, 5, 7, 2, 9], |n| *n == 2),
        Some(2)
    );

    // Predicate matches the first element only
    assert_eq!(find_exactly_one(vec![1, 2, 3], |n| *n == 1), Some(1));

    // Predicate matches the last element only
    assert_eq!(find_exactly_one(vec![1, 2, 3], |n| *n == 3), Some(3));
}
