//! ViewExt layout + paint modifiers, verified via the recording backend.

use rax_core::Color;
use rax_dom::{Attribute, Host, Mutation, RecordingBackend, Tree};
use rax_view::{text, View, ViewExt};

fn build<V: View>(
    view: V,
) -> (
    Tree,
    std::rc::Rc<std::cell::RefCell<Vec<Mutation>>>,
    rax_dom::WidgetId,
) {
    let backend = RecordingBackend::new();
    let log = backend.log();
    let mut tree = Tree::new(Host::new(backend));
    let id = view.build(&mut tree);
    (tree, log, id)
}

#[test]
fn layout_modifiers_set_style_fields() {
    let (tree, _log, id) = build(text("x").size(50.0, 30.0).flex_grow(2.0).margin(8.0));
    let style = tree.style_of(id).unwrap();
    assert_eq!(style.width, rax_core::Dimension::Points(50.0));
    assert_eq!(style.height, rax_core::Dimension::Points(30.0));
    assert_eq!(style.flex_grow, 2.0);
    assert_eq!(style.margin, rax_core::EdgeInsets::all(8.0));
}

#[test]
fn paint_modifiers_emit_attributes() {
    let (_tree, log, id) = build(
        text("x")
            .background(Color::WHITE)
            .border(2.0, Color::BLACK)
            .opacity(0.5),
    );
    let muts = log.borrow();
    let has = |a: Attribute| muts.contains(&Mutation::SetAttribute { id, attr: a });
    assert!(has(Attribute::BackgroundColor(Color::WHITE)));
    assert!(has(Attribute::BorderWidth(2.0)));
    assert!(has(Attribute::BorderColor(Color::BLACK)));
    assert!(has(Attribute::Opacity(0.5)));
}

#[test]
fn reactive_opacity_re_emits_on_signal_change() {
    use rax_reactive::create_signal;
    let o = create_signal(1.0_f32);
    let (_tree, log, id) = build(text("x").opacity_fn(move || o.get()));

    assert!(log.borrow().contains(&Mutation::SetAttribute {
        id,
        attr: Attribute::Opacity(1.0)
    }));

    log.borrow_mut().clear();
    o.set(0.5);
    assert!(
        log.borrow().contains(&Mutation::SetAttribute {
            id,
            attr: Attribute::Opacity(0.5)
        }),
        "opacity re-emitted reactively"
    );
}

#[test]
fn modifiers_chain_and_accumulate() {
    // Chaining several modifiers preserves all of them.
    let (tree, log, id) = build(text("x").width(100.0).height(40.0).corner_radius(6.0));
    let style = tree.style_of(id).unwrap();
    assert_eq!(style.width, rax_core::Dimension::Points(100.0));
    assert_eq!(style.height, rax_core::Dimension::Points(40.0));
    assert!(log.borrow().contains(&Mutation::SetAttribute {
        id,
        attr: Attribute::CornerRadius(6.0)
    }));
}

#[test]
fn accessibility_label_and_role_emit_attributes() {
    use rax_view::Role;
    let (_tree, log, id) = build(
        text("Save")
            .accessibility_label("Save document")
            .role(Role::Button),
    );
    let muts = log.borrow();
    let has = |a: Attribute| muts.contains(&Mutation::SetAttribute { id, attr: a });
    assert!(has(Attribute::AccessibilityLabel("Save document".into())));
    assert!(has(Attribute::AccessibilityRole(rax_dom::Role::Button)));
}

#[test]
fn transform_emits_affine_attribute() {
    use rax_view::Transform;
    let (_tree, log, id) = build(text("spin").transform(Transform::IDENTITY.rotate(1.0).scale(2.0)));
    assert!(log.borrow().iter().any(|m| matches!(
        m,
        Mutation::SetAttribute { id: i, attr: Attribute::Transform(t) }
            if *i == id && t.rotate == 1.0 && t.scale_x == 2.0 && t.scale_y == 2.0
    )));
}

#[test]
fn gradient_emits_linear_gradient_attribute() {
    use rax_view::LinearGradient;
    let g = LinearGradient::vertical([Color::rgb(255, 0, 0), Color::rgb(0, 0, 255)]);
    let (_tree, log, id) = build(text("hdr").gradient(g));
    assert!(log.borrow().iter().any(|m| matches!(
        m,
        Mutation::SetAttribute { id: i, attr: Attribute::Gradient(g) }
            if *i == id && g.colors.len() == 2 && g.start == (0.5, 0.0) && g.end == (0.5, 1.0)
    )));
}
