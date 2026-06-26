//! Context (provide/use) over the ownership tree.

use raxon_reactive::*;

#[derive(Clone, PartialEq, Debug)]
struct Theme(&'static str);

#[test]
fn use_context_finds_value_from_enclosing_scope() {
    let (found, scope) = create_root(|| {
        provide_context(Theme("dark"));
        // a nested scope inherits the outer context
        let (inner, inner_scope) = create_root(use_context::<Theme>);
        inner_scope.dispose();
        inner
    });
    assert_eq!(found, Some(Theme("dark")));
    scope.dispose();
}

#[test]
fn nearest_provider_wins() {
    let (found, scope) = create_root(|| {
        provide_context(Theme("outer"));
        let (inner, inner_scope) = create_root(|| {
            provide_context(Theme("inner"));
            use_context::<Theme>()
        });
        inner_scope.dispose();
        inner
    });
    assert_eq!(found, Some(Theme("inner")));
    scope.dispose();
}

#[test]
fn missing_context_is_none() {
    let (found, scope) = create_root(use_context::<Theme>);
    assert_eq!(found, None);
    scope.dispose();
}

#[test]
fn context_drives_an_effect() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let log = Rc::new(RefCell::new(Vec::<&'static str>::new()));
    let log2 = log.clone();
    let (_, scope) = create_root(move || {
        provide_context(Theme("brand"));
        create_effect(move || {
            if let Some(Theme(name)) = use_context::<Theme>() {
                log2.borrow_mut().push(name);
            }
        });
    });
    assert_eq!(*log.borrow(), vec!["brand"]);
    scope.dispose();
}
