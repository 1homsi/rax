use std::cell::RefCell;
use std::rc::Rc;

use crate::reactive::{create_effect, create_root};

use super::{provide_theme, theme, Theme};

#[test]
fn theme_is_readable_from_context() {
    let (primary, scope) = create_root(|| {
        provide_theme(Theme::light());
        theme().colors.primary
    });
    assert_eq!(primary, Theme::light().colors.primary);
    scope.dispose();
}

#[test]
fn switching_theme_reactively_updates_readers() {
    let log: Rc<RefCell<Vec<crate::core::Color>>> = Rc::new(RefCell::new(Vec::new()));
    let log2 = log.clone();

    let (handle, scope) = create_root(move || {
        let t = provide_theme(Theme::light());
        create_effect(move || log2.borrow_mut().push(theme().colors.background));
        t
    });

    assert_eq!(log.borrow()[0], Theme::light().colors.background);

    handle.set(Theme::dark());
    assert_eq!(
        *log.borrow().last().unwrap(),
        Theme::dark().colors.background
    );

    scope.dispose();
}

#[test]
fn default_theme_is_provided_when_missing() {
    let (bg, scope) = create_root(|| theme().colors.background);
    assert_eq!(bg, Theme::light().colors.background);
    scope.dispose();
}
