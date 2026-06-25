//! Navigation behaviour, driven through the headless test harness.

use rax_nav::{create_navigator, routes, use_navigator, Navigator};
use rax_test::TestHarness;
use rax_view::{boxed, button, column, text, View};

#[derive(Clone, PartialEq)]
enum Screen {
    Home,
    Details(u32),
}

fn app() -> impl View {
    let nav = create_navigator(Screen::Home);
    routes(nav, move |screen| match screen {
        Screen::Home => boxed(column((
            text("Home"),
            button("Open", move || nav.push(Screen::Details(7))),
        ))),
        Screen::Details(id) => boxed(column((
            text(move || format!("Details {id}")),
            button("Back", move || nav.pop()),
        ))),
    })
}

#[test]
fn push_and_pop_swap_screens() {
    let mut ui = TestHarness::mount(app);
    ui.assert_text("Home");
    assert!(ui.find_text("Details 7").is_none());

    let open = ui.find_button("Open").unwrap();
    ui.tap(open);
    ui.assert_text("Details 7");
    assert!(ui.find_text("Home").is_none(), "home screen torn down");

    let back = ui.find_button("Back").unwrap();
    ui.tap(back);
    ui.assert_text("Home");
}

#[test]
fn navigator_depth_and_can_pop() {
    let nav: Navigator<Screen> = create_navigator(Screen::Home);
    assert_eq!(nav.depth(), 1);
    assert!(!nav.can_pop());
    nav.push(Screen::Details(1));
    assert_eq!(nav.depth(), 2);
    assert!(nav.can_pop());
    nav.pop();
    assert_eq!(nav.depth(), 1);
}

#[test]
fn use_navigator_reaches_the_provided_navigator() {
    use std::cell::Cell;
    use std::rc::Rc;

    let found = Rc::new(Cell::new(false));
    let found2 = found.clone();
    // The factory runs inside the app's root scope, so context is live.
    let ui = TestHarness::mount(move || {
        let _nav = create_navigator(Screen::Home);
        found2.set(use_navigator::<Screen>().is_some());
        column((text("ok"),))
    });
    assert!(found.get(), "use_navigator found the provided navigator");
    ui.assert_text("ok");
}
