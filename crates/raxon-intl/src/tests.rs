use std::cell::RefCell;
use std::rc::Rc;

use raxon_reactive::{create_effect, create_root};

use super::{provide_locale, t, t_args, Catalog};

#[test]
fn lookup_and_fallback() {
    let (_, scope) = create_root(|| {
        provide_locale(Catalog::from([("greeting", "Hello")]));
        assert_eq!(t("greeting"), "Hello");
        assert_eq!(t("missing"), "missing", "falls back to the key");
    });
    scope.dispose();
}

#[test]
fn interpolation_substitutes_args() {
    let (_, scope) = create_root(|| {
        provide_locale(Catalog::new().with("hi", "Hi {name}, you have {n} messages"));
        assert_eq!(
            t_args("hi", &[("name", "Sam"), ("n", "3")]),
            "Hi Sam, you have 3 messages"
        );
    });
    scope.dispose();
}

#[test]
fn switching_locale_reactively_updates_readers() {
    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let log2 = log.clone();
    let (loc, scope) = create_root(move || {
        let loc = provide_locale(Catalog::from([("hi", "Hello")]));
        create_effect(move || log2.borrow_mut().push(t("hi")));
        loc
    });

    assert_eq!(log.borrow()[0], "Hello");
    loc.set(Catalog::from([("hi", "Hola")]));
    assert_eq!(*log.borrow().last().unwrap(), "Hola");
    scope.dispose();
}

#[test]
fn plural_selects_one_vs_other_and_interpolates_count() {
    use super::{provide_locale, t_plural, Catalog};
    use raxon_reactive::create_root;
    let (_, scope) = create_root(|| {
        provide_locale(Catalog::from([
            ("items.one", "{count} item"),
            ("items.other", "{count} items"),
        ]));
        assert_eq!(t_plural("items", 1), "1 item");
        assert_eq!(t_plural("items", 0), "0 items");
        assert_eq!(t_plural("items", 7), "7 items");
    });
    scope.dispose();
}
