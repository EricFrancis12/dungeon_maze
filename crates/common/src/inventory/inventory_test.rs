use crate::inventory::item::{Item, ItemName};
use strum::IntoEnumIterator;

#[test]
fn test_item_merge_with_no_overflow() {
    for item_name in ItemName::iter() {
        let mut item_1 = Item::new(item_name.clone(), 1);
        let item_2 = Item::new(item_name, 8);

        let rem_item = item_1.merge(item_2);

        if item_name._is_stackable() {
            assert_eq!(item_1.amt, 9);
            assert_eq!(rem_item.is_none(), true);
        } else {
            assert_eq!(item_1.amt, 1);
            assert_eq!(rem_item.is_some(), true);
            assert_eq!(rem_item.unwrap().amt, 8);
        }
    }
}

#[test]
fn test_item_merge_with_overflow() {
    for item_name in ItemName::iter() {
        let mut item_1 = Item::new(item_name.clone(), 40);
        let ma = item_1.max_amt();
        let item_2 = Item::new(item_name.clone(), ma);

        let rem_item = item_1.merge(item_2);

        assert_eq!(item_1.amt, ma);
        assert_eq!(rem_item, Some(Item::new(item_name, 40)));
    }
}

#[test]
fn test_item_merge_with_diff_names() {
    let mut item_1 = Item::new(ItemName::Coal, 20);
    let item_2 = Item::new(ItemName::Cotton, 10);

    let rem_item = item_1.merge(item_2);

    assert_eq!(item_1.amt, 20);
    assert_eq!(rem_item, Some(Item::new(ItemName::Cotton, 10)));
}
