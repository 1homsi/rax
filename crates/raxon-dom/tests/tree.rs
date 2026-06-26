//! Element-tree structure and the reactive mutation stream.

mod common;
use common::harness;

use raxon_dom::*;
use raxon_reactive::create_signal;

#[test]
fn create_emits_create_mutation() {
    let (mut tree, log) = harness();
    let v = tree.create_view();
    assert_eq!(
        *log.borrow(),
        vec![Mutation::Create {
            id: v,
            kind: WidgetKind::View
        }]
    );
    assert_eq!(tree.len(), 1);
}

#[test]
fn static_attribute_emits_once() {
    let (mut tree, log) = harness();
    let t = tree.create_text();
    tree.set(t, Attribute::Text("hi".into()));
    assert_eq!(
        *log.borrow(),
        vec![
            Mutation::Create {
                id: t,
                kind: WidgetKind::Text
            },
            Mutation::SetAttribute {
                id: t,
                attr: Attribute::Text("hi".into())
            },
        ]
    );
}

#[test]
fn append_records_child_and_index() {
    let (mut tree, log) = harness();
    let root = tree.create_view();
    let a = tree.create_text();
    let b = tree.create_text();
    tree.append(root, a);
    tree.append(root, b);

    assert_eq!(tree.children_of(root), &[a, b]);
    let muts = log.borrow();
    assert_eq!(
        muts[muts.len() - 2],
        Mutation::InsertChild {
            parent: root,
            index: 0,
            child: a
        }
    );
    assert_eq!(
        muts[muts.len() - 1],
        Mutation::InsertChild {
            parent: root,
            index: 1,
            child: b
        }
    );
}

#[test]
fn reactive_bind_emits_exactly_one_mutation_per_change() {
    let (mut tree, log) = harness();
    let count = create_signal(0);
    let label = tree.create_text();
    tree.bind(label, move || {
        Attribute::Text(format!("Count: {}", count.get()))
    });

    assert_eq!(
        *log.borrow(),
        vec![
            Mutation::Create {
                id: label,
                kind: WidgetKind::Text
            },
            Mutation::SetAttribute {
                id: label,
                attr: Attribute::Text("Count: 0".into())
            },
        ]
    );

    log.borrow_mut().clear();
    count.set(1);
    count.set(2);

    assert_eq!(
        *log.borrow(),
        vec![
            Mutation::SetAttribute {
                id: label,
                attr: Attribute::Text("Count: 1".into())
            },
            Mutation::SetAttribute {
                id: label,
                attr: Attribute::Text("Count: 2".into())
            },
        ]
    );
}

#[test]
fn remove_tears_down_subtree_children_first_and_disposes_bindings() {
    let (mut tree, log) = harness();
    let count = create_signal(0);
    let root = tree.create_view();
    let child = tree.create_text();
    tree.bind(child, move || Attribute::Text(count.get().to_string()));
    tree.append(root, child);

    log.borrow_mut().clear();
    tree.remove(root);

    assert_eq!(
        *log.borrow(),
        vec![
            Mutation::Destroy { id: child },
            Mutation::Destroy { id: root }
        ]
    );
    assert!(tree.is_empty());

    log.borrow_mut().clear();
    count.set(99);
    assert!(log.borrow().is_empty(), "disposed binding must not emit");
}

#[test]
fn remove_child_subtree_emits_remove_then_destroy() {
    let (mut tree, log) = harness();
    let root = tree.create_view();
    let child = tree.create_view();
    let grandchild = tree.create_text();
    tree.append(root, child);
    tree.append(child, grandchild);

    log.borrow_mut().clear();
    tree.remove(child);

    assert_eq!(
        *log.borrow(),
        vec![
            Mutation::RemoveChild {
                parent: root,
                child
            },
            Mutation::Destroy { id: grandchild },
            Mutation::Destroy { id: child },
        ]
    );
    assert_eq!(tree.children_of(root), &[] as &[WidgetId]);
}
