use hypernote_mdx::ast::NodeTag;

#[test]
fn unchecked_checkbox() {
    let source = "- [ ] Task one\n- [ ] Task two\n";
    let ast = hypernote_mdx::parse(source);

    let list_items: Vec<_> = ast
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.tag == NodeTag::ListItem)
        .collect();
    assert_eq!(2, list_items.len());

    for &(i, _) in &list_items {
        let info = ast.list_item_info(i as u32);
        assert_eq!(Some(false), info.checked);
    }
}

#[test]
fn checked_checkbox() {
    let source = "- [x] Done task\n";
    let ast = hypernote_mdx::parse(source);

    let item_idx = ast
        .nodes
        .iter()
        .enumerate()
        .find(|(_, n)| n.tag == NodeTag::ListItem)
        .map(|(i, _)| i as u32)
        .unwrap();

    let info = ast.list_item_info(item_idx);
    assert_eq!(Some(true), info.checked);
}

#[test]
fn checked_uppercase_x() {
    let source = "- [X] Also done\n";
    let ast = hypernote_mdx::parse(source);

    let item_idx = ast
        .nodes
        .iter()
        .enumerate()
        .find(|(_, n)| n.tag == NodeTag::ListItem)
        .map(|(i, _)| i as u32)
        .unwrap();

    let info = ast.list_item_info(item_idx);
    assert_eq!(Some(true), info.checked);
}

#[test]
fn mixed_checked_unchecked() {
    let source = "- [x] Done\n- [ ] Not done\n- [X] Also done\n";
    let ast = hypernote_mdx::parse(source);

    let list_items: Vec<_> = ast
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.tag == NodeTag::ListItem)
        .map(|(i, _)| i as u32)
        .collect();
    assert_eq!(3, list_items.len());

    assert_eq!(Some(true), ast.list_item_info(list_items[0]).checked);
    assert_eq!(Some(false), ast.list_item_info(list_items[1]).checked);
    assert_eq!(Some(true), ast.list_item_info(list_items[2]).checked);

    // All items should be in a single unordered list
    let list_count = ast
        .nodes
        .iter()
        .filter(|n| n.tag == NodeTag::ListUnordered)
        .count();
    assert_eq!(1, list_count);
}

#[test]
fn regular_list_item_no_checkbox() {
    let source = "- Regular item\n";
    let ast = hypernote_mdx::parse(source);

    let item_idx = ast
        .nodes
        .iter()
        .enumerate()
        .find(|(_, n)| n.tag == NodeTag::ListItem)
        .map(|(i, _)| i as u32)
        .unwrap();

    let info = ast.list_item_info(item_idx);
    assert_eq!(None, info.checked);
}

#[test]
fn ordered_list_checkbox() {
    let source = "1. [ ] First\n2. [x] Second\n";
    let ast = hypernote_mdx::parse(source);

    let list_items: Vec<_> = ast
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.tag == NodeTag::ListItem)
        .map(|(i, _)| i as u32)
        .collect();
    assert_eq!(2, list_items.len());

    assert_eq!(Some(false), ast.list_item_info(list_items[0]).checked);
    assert_eq!(Some(true), ast.list_item_info(list_items[1]).checked);

    let list_count = ast
        .nodes
        .iter()
        .filter(|n| n.tag == NodeTag::ListOrdered)
        .count();
    assert_eq!(1, list_count);
}

#[test]
fn checkbox_roundtrip() {
    let source = "- [ ] Unchecked\n- [x] Checked\n";
    let ast = hypernote_mdx::parse(source);
    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn checkbox_roundtrip_ordered() {
    let source = "1. [ ] First\n2. [x] Second\n";
    let ast = hypernote_mdx::parse(source);
    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn checkbox_json_serialization() {
    let source = "- [ ] Unchecked\n- [x] Checked\n";
    let ast = hypernote_mdx::parse(source);
    let json = hypernote_mdx::serialize_tree(&ast);

    assert!(
        json.contains("\"checked\":false"),
        "JSON should contain checked:false, got: {}",
        json
    );
    assert!(
        json.contains("\"checked\":true"),
        "JSON should contain checked:true, got: {}",
        json
    );
}

#[test]
fn regular_list_json_no_checked_field() {
    let source = "- Regular item\n";
    let ast = hypernote_mdx::parse(source);
    let json = hypernote_mdx::serialize_tree(&ast);

    assert!(
        json.contains("\"checked\":null"),
        "Regular list items should have checked:null for schema consistency, got: {}",
        json
    );
}

#[test]
fn checkbox_at_end_of_line() {
    let source = "- [ ]\n- [x]\n";
    let ast = hypernote_mdx::parse(source);

    let list_items: Vec<_> = ast
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.tag == NodeTag::ListItem)
        .map(|(i, _)| i as u32)
        .collect();
    assert_eq!(2, list_items.len());

    assert_eq!(Some(false), ast.list_item_info(list_items[0]).checked);
    assert_eq!(Some(true), ast.list_item_info(list_items[1]).checked);
}

#[test]
fn not_a_checkbox_missing_space_inside() {
    let source = "- [] text\n";
    let ast = hypernote_mdx::parse(source);

    let item_idx = ast
        .nodes
        .iter()
        .enumerate()
        .find(|(_, n)| n.tag == NodeTag::ListItem)
        .map(|(i, _)| i as u32)
        .unwrap();

    let info = ast.list_item_info(item_idx);
    assert_eq!(
        None, info.checked,
        "[] without space inside is not a checkbox"
    );
}

#[test]
fn not_a_checkbox_invalid_char() {
    let source = "- [y] text\n";
    let ast = hypernote_mdx::parse(source);

    let item_idx = ast
        .nodes
        .iter()
        .enumerate()
        .find(|(_, n)| n.tag == NodeTag::ListItem)
        .map(|(i, _)| i as u32)
        .unwrap();

    let info = ast.list_item_info(item_idx);
    assert_eq!(None, info.checked, "[y] is not a valid checkbox");
}

#[test]
fn checkbox_preserves_inline_formatting() {
    let source = "- [ ] **bold** task\n";
    let ast = hypernote_mdx::parse(source);

    let item_idx = ast
        .nodes
        .iter()
        .enumerate()
        .find(|(_, n)| n.tag == NodeTag::ListItem)
        .map(|(i, _)| i as u32)
        .unwrap();

    let info = ast.list_item_info(item_idx);
    assert_eq!(Some(false), info.checked);

    // Should have children including a Strong node
    let children = ast.children(item_idx);
    let has_strong = children
        .iter()
        .any(|&c| ast.nodes[c as usize].tag == NodeTag::Strong);
    assert!(
        has_strong,
        "Checkbox list item should preserve inline formatting"
    );
}

#[test]
fn checkbox_double_roundtrip() {
    let source = "- [ ] Unchecked\n- [x] Checked\n- Regular\n";
    let ast1 = hypernote_mdx::parse(source);
    let rendered1 = hypernote_mdx::render(&ast1);
    let ast2 = hypernote_mdx::parse(&rendered1);
    let rendered2 = hypernote_mdx::render(&ast2);
    assert_eq!(rendered1, rendered2, "Double round-trip should be stable");
}
