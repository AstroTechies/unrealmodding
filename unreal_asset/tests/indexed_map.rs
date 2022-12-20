use std::collections::HashMap;

use unreal_asset::containers::indexed_map::IndexedMap;

#[test]
fn insertion() {
    let mut indexed_map = IndexedMap::new();

    indexed_map.insert("Test1".to_string(), "Value1".to_string());
    indexed_map.insert("Test2".to_string(), "Value2".to_string());
    indexed_map.insert("Test3".to_string(), "Value3".to_string());
    indexed_map.insert("Test4".to_string(), "Value4".to_string());

    assert_eq!(indexed_map.get_by_key("Test1"), Some(&"Value1".to_string()));
    assert_eq!(indexed_map.get_by_key("Test2"), Some(&"Value2".to_string()));
    assert_eq!(indexed_map.get_by_key("Test3"), Some(&"Value3".to_string()));
    assert_eq!(indexed_map.get_by_key("Test4"), Some(&"Value4".to_string()));
}

#[test]
fn iteration_by_index() {
    let mut indexed_map = IndexedMap::new();

    indexed_map.insert("Test1".to_string(), 1);
    indexed_map.insert("Test2".to_string(), 2);
    indexed_map.insert("Test3".to_string(), 3);
    indexed_map.insert("Test4".to_string(), 4);

    let mut element = 0;
    for (_, _, value) in indexed_map.iter() {
        assert_eq!(*value, element + 1);
        element += 1;
    }
}

#[test]
fn iteration_by_key() {
    let mut indexed_map = IndexedMap::new();
    let mut hash_map = HashMap::new();

    indexed_map.insert("Test1".to_string(), "Value1".to_string());
    hash_map.insert("Test1".to_string(), "Value1".to_string());
    indexed_map.insert("Test2".to_string(), "Value2".to_string());
    hash_map.insert("Test2".to_string(), "Value2".to_string());
    indexed_map.insert("Test3".to_string(), "Value3".to_string());
    hash_map.insert("Test3".to_string(), "Value3".to_string());
    indexed_map.insert("Test4".to_string(), "Value4".to_string());
    hash_map.insert("Test4".to_string(), "Value4".to_string());

    for (_, key, value) in indexed_map.iter_key() {
        assert_eq!(&hash_map[key], value)
    }
}

#[test]
fn removal() {
    let mut indexed_map = IndexedMap::new();
    let mut hash_map = HashMap::new();

    indexed_map.insert("Test1".to_string(), "Value1".to_string());
    hash_map.insert("Test1".to_string(), "Value1".to_string());
    indexed_map.insert("Test2".to_string(), "Value2".to_string());
    hash_map.insert("Test2".to_string(), "Value2".to_string());
    indexed_map.insert("Test3".to_string(), "Value3".to_string());
    hash_map.insert("Test3".to_string(), "Value3".to_string());
    indexed_map.insert("Test4".to_string(), "Value4".to_string());
    hash_map.insert("Test4".to_string(), "Value4".to_string());

    assert_eq!(indexed_map.len(), hash_map.len());

    for (_, key, value) in indexed_map.iter_key() {
        assert_eq!(&hash_map[key], value)
    }

    indexed_map.remove_by_key("Test1");
    hash_map.remove("Test1");

    assert_eq!(indexed_map.len(), hash_map.len());

    for (_, key, value) in indexed_map.iter_key() {
        assert_eq!(&hash_map[key], value)
    }
}
