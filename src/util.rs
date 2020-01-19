use std::{cell::*, rc::*};
use cursive::{view::*, views::*, direction::Direction};

// a wrapper of view V
// it allows you to hold an mutable reference to a view after adding it to another view
pub struct RcView<V> { inner: Rc<RefCell<V>> }

impl<V> RcView<V> {
  pub fn new(v: V) -> Self { Self { inner: Rc::new(RefCell::new(v)) } }

  pub fn get(&self) -> RefMut<V> { self.inner.borrow_mut() }

  pub fn clone(&self) -> Self { Self { inner: self.inner.clone() } }
}

// basically copied from NamedView
impl<V: View> ViewWrapper for RcView<V> {
  type V = V;

  fn with_view<F, R>(&self, f: F) -> Option<R> where F: FnOnce(&Self::V) -> R {
    self.inner.try_borrow().ok().map(|v| f(&*v))
  }

  fn with_view_mut<F, R>(&mut self, f: F) -> Option<R> where F: FnOnce(&mut Self::V) -> R {
    self.inner.try_borrow_mut().ok().map(|mut v| f(&mut *v))
  }

  fn into_inner(mut self) -> Result<Self::V, Self> where Self::V: Sized {
    match Rc::try_unwrap(self.inner) {
      Err(rc) => {
        self.inner = rc;
        Err(self)
      }
      Ok(cell) => Ok(cell.into_inner()),
    }
  }
}

// `TextView` doesn't take focus, while `FocusableTextView` does
pub struct FocusableTextView { inner: TextView }

impl FocusableTextView {
  pub fn new(t: TextView) -> Self { Self { inner: t } }

  pub fn get(&mut self) -> &mut TextView { &mut self.inner }
}

impl ViewWrapper for FocusableTextView {
  cursive::wrap_impl!(self.inner: TextView);

  fn wrap_take_focus(&mut self, _: Direction) -> bool { true }
}

// `ScrollView` scrolls to the focused location of inner view at mouse event automatically,
// while NonAutoScrollView doesn't
pub struct NonAutoScrollView<V> { inner: ScrollView<V> }

impl<V: View> NonAutoScrollView<V> {
  pub fn new(v: V) -> Self { Self { inner: ScrollView::new(v) } }

  pub fn inner(&mut self) -> &mut V { self.inner.get_inner_mut() }
}

impl<V: View> ViewWrapper for NonAutoScrollView<V> {
  cursive::wrap_impl!(self.inner: ScrollView<V>);

  fn wrap_take_focus(&mut self, source: Direction) -> bool {
    // todo: the implementation of ScrollView::take_focus also uses `Core::is_scrolling` to judge, is this necessary?
    self.inner().take_focus(source)
  }

  fn wrap_focus_view(&mut self, selector: &Selector<'_>) -> Result<(), ()> {
    self.inner().focus_view(selector)
  }
}