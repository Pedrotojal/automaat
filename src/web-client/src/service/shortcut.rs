//! The shortcut service can act on keyboard input to perform specific actions.
//!
//! In a sense, this is closely related to the `Router`, whereas the router acts
//! on path changes and updates the models, this service acts on keystrokes and
//! updates the models.

use crate::component::Navbar;
use crate::controller::Controller;
use crate::model::task;
use crate::router::Route;
use crate::utils;
use dodrio::VdomWeak;
use futures::prelude::*;
use gloo_events::{EventListener, EventListenerOptions};
use std::marker::PhantomData;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlElement, HtmlInputElement, KeyboardEvent};

/// The Enter key code.
pub(crate) const ENTER: u32 = 13;

/// The Escape key code.
pub(crate) const ESCAPE: u32 = 27;

/// The F key code.
pub(crate) const F: u32 = 70;

/// The Shortcut service.
// TODO: think about a different approach:
//
// The `ShortcutService` keeps a hashmap of (key, [actions]), it then exposes a
// `register_shortcut` method that takes a key, and a future.
//
// When a key is pressed, it executes any actions it knows for that given key.
//
// Now, where/how do we register these shortcuts? In the components? If so, then
// the shortcut service would have to be accessible on `App` (doable, nothing
// wrong with that). But are components the right place?
//
// I feel like they are? Since shortcuts are tied to what is visible on the
// screen?
//
// But we actually don't have access to `App` in these components, only in the
// events attached to the DOM (using `on`). This makes sense, since these
// components are potentially rendered each frame, so doing that would be
// expensive...
//
// OTOH, which component is shown is driven by the state of the models, so then
// perhaps the models themselves should be responsible for registering
// shortcuts?
//
// But driving that point to its logical conclusion (maybe?); the controller is
// responsible for managing the model state, so perhaps _it_ should be
// responsible for the shortcuts as well? Although I don't think that makes a
// lot of sense, as it would have to know all shortcuts for all models in a
// single place, whereas it is more logical for that knowledge to be hidden in
// the models themselves...
//
// But, we don't have access to `App` or the `ShortcutService` in the models
// either, and I don't think it makes sense to pass that into each model as a
// dependency? Or maybe it does? It does reflect some IoC semantics... That
// would require putting it behind a `RefCell` or `Mutex` though, I suspect.
#[derive(Default)]
pub(crate) struct Service<C = Controller>(PhantomData<C>);

impl<C> Service<C>
where
    C: task::Actions,
{
    /// Listen for keyboard input and perform model or DOM updates based on the
    /// input.
    pub(crate) fn listen(&self, vdom: VdomWeak) {
        use Route::*;

        let options = EventListenerOptions::enable_prevent_default();
        EventListener::new_with_options(&utils::document(), "keydown", options, move |event| {
            let event = event.unchecked_ref::<KeyboardEvent>();
            let target = event.target().unwrap_throw();
            let route = match Route::active() {
                None => return,
                Some(route) => route,
            };

            match route {
                Home => {
                    let navbar = Navbar::<C>::new();
                    match event.key_code() {
                        F if !target.has_type::<HtmlInputElement>() => navbar.focus_search(),
                        ESCAPE => navbar.blur_search(),
                        _ => return,
                    };
                }
                Task(_) => match event.key_code() {
                    ESCAPE if !target.has_type::<HtmlInputElement>() => spawn_local(
                        vdom.with_component({
                            let vdom = vdom.clone();
                            |root| C::close_active_task(root, vdom)
                        })
                        .map_err(|_| ()),
                    ),
                    ENTER => {
                        utils::element::<HtmlElement>(".task-details button[type=submit]").click()
                    }
                    _ => return,
                },
            }

            event.prevent_default();
        })
        .forget();
    }
}
