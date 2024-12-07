use crate::inventory::{Item, ItemName};

use strum::IntoEnumIterator;

#[test]
fn test_item_merge_with_no_overflow() {
    for item_name in ItemName::iter() {
        let mut item1 = Item::new(item_name.clone(), 40);
        let item2 = Item::new(item_name, 8);

        let rem_item = item1.merge(item2);

        assert_eq!(item1.amt, 48);
        assert_eq!(rem_item, None);
    }
}

#[test]
fn test_item_merge_with_overflow() {
    for item_name in ItemName::iter() {
        let mut item1 = Item::new(item_name.clone(), 40);
        let ma = item1.max_amt();
        let item2 = Item::new(item_name.clone(), ma);

        let rem_item = item1.merge(item2);

        assert_eq!(item1.amt, ma);
        assert_eq!(rem_item, Some(Item::new(item_name, 40)));
    }
}

#[test]
fn test_item_merge_with_diff_names() {
    let mut item1 = Item::new(ItemName::Coal, 20);
    let item2 = Item::new(ItemName::Cotton, 10);

    let rem_item = item1.merge(item2);

    assert_eq!(item1.amt, 20);
    assert_eq!(rem_item, Some(Item::new(ItemName::Cotton, 10)));
}
