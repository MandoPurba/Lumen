use kireiui::{
    Alignment, Constraints, HAlign, HStack, SizedBox, VAlign, VStack, View, ViewExt, ViewTree,
    ZStack,
};

// ---------------------------------------------------------------------------
// SizedBox
// ---------------------------------------------------------------------------

#[test]
fn sized_box_intrinsic_size() {
    let view = SizedBox::new(100.0, 50.0);
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let r = tree.rect(root);
    assert_eq!(r.width, 100.0);
    assert_eq!(r.height, 50.0);
}

#[test]
fn sized_box_clamped_to_constraints() {
    let view = SizedBox::new(300.0, 200.0);
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::exact(100.0, 80.0));

    let r = tree.rect(root);
    assert_eq!(r.width, 100.0);
    assert_eq!(r.height, 80.0);
}

// ---------------------------------------------------------------------------
// VStack
// ---------------------------------------------------------------------------

#[test]
fn vstack_stacks_vertically_with_spacing() {
    let view = VStack::new()
        .spacing(10.0)
        .child(SizedBox::new(100.0, 50.0))
        .child(SizedBox::new(200.0, 30.0));
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let r = tree.rect(root);
    assert_eq!(r.width, 200.0);
    assert_eq!(r.height, 90.0); // 50 + 10 + 30

    let kids = tree.children(root);
    assert_eq!(tree.rect(kids[0]).y, 0.0);
    assert_eq!(tree.rect(kids[1]).y, 60.0); // 50 + 10
}

#[test]
fn vstack_center_alignment() {
    let view = VStack::new()
        .alignment(HAlign::Center)
        .child(SizedBox::new(100.0, 20.0))
        .child(SizedBox::new(200.0, 20.0));
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let kids = tree.children(root);
    assert_eq!(tree.rect(kids[0]).x, 50.0); // (200 - 100) * 0.5
    assert_eq!(tree.rect(kids[1]).x, 0.0);
}

#[test]
fn vstack_leading_alignment() {
    let view = VStack::new()
        .alignment(HAlign::Leading)
        .child(SizedBox::new(100.0, 20.0))
        .child(SizedBox::new(200.0, 20.0));
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let kids = tree.children(root);
    assert_eq!(tree.rect(kids[0]).x, 0.0);
    assert_eq!(tree.rect(kids[1]).x, 0.0);
}

#[test]
fn vstack_trailing_alignment() {
    let view = VStack::new()
        .alignment(HAlign::Trailing)
        .child(SizedBox::new(100.0, 20.0))
        .child(SizedBox::new(200.0, 20.0));
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let kids = tree.children(root);
    assert_eq!(tree.rect(kids[0]).x, 100.0); // (200 - 100) * 1.0
    assert_eq!(tree.rect(kids[1]).x, 0.0);
}

// ---------------------------------------------------------------------------
// HStack
// ---------------------------------------------------------------------------

#[test]
fn hstack_stacks_horizontally_with_spacing() {
    let view = HStack::new()
        .spacing(8.0)
        .child(SizedBox::new(100.0, 50.0))
        .child(SizedBox::new(60.0, 30.0));
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let r = tree.rect(root);
    assert_eq!(r.width, 168.0); // 100 + 8 + 60
    assert_eq!(r.height, 50.0);

    let kids = tree.children(root);
    assert_eq!(tree.rect(kids[0]).x, 0.0);
    assert_eq!(tree.rect(kids[1]).x, 108.0); // 100 + 8
}

#[test]
fn hstack_vertical_alignment() {
    let view = HStack::new()
        .alignment(VAlign::Top)
        .child(SizedBox::new(50.0, 100.0))
        .child(SizedBox::new(50.0, 40.0));
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let kids = tree.children(root);
    assert_eq!(tree.rect(kids[0]).y, 0.0);
    assert_eq!(tree.rect(kids[1]).y, 0.0); // top-aligned
}

#[test]
fn hstack_center_vertical() {
    let view = HStack::new()
        .alignment(VAlign::Center)
        .child(SizedBox::new(50.0, 100.0))
        .child(SizedBox::new(50.0, 40.0));
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let kids = tree.children(root);
    assert_eq!(tree.rect(kids[1]).y, 30.0); // (100 - 40) * 0.5
}

// ---------------------------------------------------------------------------
// ZStack
// ---------------------------------------------------------------------------

#[test]
fn zstack_overlays_with_center_alignment() {
    let view = ZStack::new()
        .child(SizedBox::new(200.0, 100.0))
        .child(SizedBox::new(50.0, 30.0));
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let r = tree.rect(root);
    assert_eq!(r.width, 200.0);
    assert_eq!(r.height, 100.0);

    let kids = tree.children(root);
    assert_eq!(tree.rect(kids[0]).x, 0.0);
    assert_eq!(tree.rect(kids[0]).y, 0.0);
    assert_eq!(tree.rect(kids[1]).x, 75.0); // (200 - 50) * 0.5
    assert_eq!(tree.rect(kids[1]).y, 35.0); // (100 - 30) * 0.5
}

#[test]
fn zstack_top_leading_alignment() {
    let view = ZStack::new()
        .alignment(Alignment::TOP_LEADING)
        .child(SizedBox::new(200.0, 100.0))
        .child(SizedBox::new(50.0, 30.0));
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let kids = tree.children(root);
    assert_eq!(tree.rect(kids[1]).x, 0.0);
    assert_eq!(tree.rect(kids[1]).y, 0.0);
}

// ---------------------------------------------------------------------------
// Padding
// ---------------------------------------------------------------------------

#[test]
fn padding_insets_child() {
    let view = SizedBox::new(100.0, 50.0).padding(10.0);
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let r = tree.rect(root);
    assert_eq!(r.width, 120.0);
    assert_eq!(r.height, 70.0);

    let kids = tree.children(root);
    assert_eq!(tree.rect(kids[0]).x, 10.0);
    assert_eq!(tree.rect(kids[0]).y, 10.0);
    assert_eq!(tree.rect(kids[0]).width, 100.0);
    assert_eq!(tree.rect(kids[0]).height, 50.0);
}

// ---------------------------------------------------------------------------
// Frame
// ---------------------------------------------------------------------------

#[test]
fn frame_overrides_size_and_centers_child() {
    let view = SizedBox::new(100.0, 50.0).frame_size(200.0, 200.0);
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let r = tree.rect(root);
    assert_eq!(r.width, 200.0);
    assert_eq!(r.height, 200.0);

    let kids = tree.children(root);
    assert_eq!(tree.rect(kids[0]).x, 50.0); // (200 - 100) * 0.5
    assert_eq!(tree.rect(kids[0]).y, 75.0); // (200 - 50) * 0.5
}

#[test]
fn frame_top_leading_alignment() {
    let view = SizedBox::new(60.0, 40.0).frame(Some(200.0), Some(200.0), Alignment::TOP_LEADING);
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let kids = tree.children(root);
    assert_eq!(tree.rect(kids[0]).x, 0.0);
    assert_eq!(tree.rect(kids[0]).y, 0.0);
}

// ---------------------------------------------------------------------------
// Nested stacks
// ---------------------------------------------------------------------------

#[test]
fn nested_vstack_hstack() {
    let view = VStack::new()
        .spacing(10.0)
        .child(
            HStack::new()
                .spacing(5.0)
                .child(SizedBox::new(40.0, 30.0))
                .child(SizedBox::new(60.0, 30.0)),
        )
        .child(SizedBox::new(80.0, 20.0));
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let r = tree.rect(root);
    assert_eq!(r.width, 105.0); // max(40+5+60, 80) = 105
    assert_eq!(r.height, 60.0); // 30 + 10 + 20

    let kids = tree.children(root);
    let hstack_rect = tree.rect(kids[0]);
    assert_eq!(hstack_rect.width, 105.0);
    assert_eq!(hstack_rect.height, 30.0);

    let hstack_kids = tree.children(kids[0]);
    assert_eq!(tree.rect(hstack_kids[0]).x, 0.0);
    assert_eq!(tree.rect(hstack_kids[1]).x, 45.0); // 40 + 5
}

#[test]
fn padding_inside_stack() {
    let view = VStack::new().child(SizedBox::new(80.0, 40.0).padding(10.0));
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let r = tree.rect(root);
    assert_eq!(r.width, 100.0); // 80 + 20
    assert_eq!(r.height, 60.0); // 40 + 20
}

#[test]
fn frame_inside_stack() {
    let view = HStack::new()
        .spacing(10.0)
        .child(SizedBox::new(50.0, 30.0).frame_size(100.0, 60.0))
        .child(SizedBox::new(50.0, 30.0));
    let mut tree = ViewTree::new();
    let root = view.build(&mut tree);
    tree.layout(root, Constraints::unbounded());

    let r = tree.rect(root);
    assert_eq!(r.width, 160.0); // 100 + 10 + 50
    assert_eq!(r.height, 60.0);
}
