#[test]
fn name_editing() {
    let mut map = unreal_asset::containers::name_map::NameMap::from_name_batch(&[
        "never".to_string(),
        "gonna".to_string(),
        "give".to_string(),
        "you".to_string(),
        "up".to_string(),
    ]);
    let never = map.get_ref().create_fname(0, 0);
    let content = never.get_owned_content();
    // force creation of new name with same content
    let i = map.get_mut().add_name_reference(content.clone(), true);
    let new = map.get_ref().create_fname(i, 0);
    assert_eq!(content, new.get_owned_content());
}
