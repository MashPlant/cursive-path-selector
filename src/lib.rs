use std::{path::*, cell::*, rc::*, time::*, io, env, fs};
use cursive::{view::*, views::*, event::*, theme::*};
use cursive::utils::markup::StyledString;

pub struct PathSelector {
  current_dir: PathBuf,
  dir_content: Vec<PathBuf>,
  last_click: Option<Instant>,
  view: LinearLayout,
  // below are reference to the views inside `view`
  content: RcView<LinearLayout>,
  err_msg: RcView<TextView>,
}

impl PathSelector {
  pub fn new() -> io::Result<Self> {
    env::current_dir().and_then(Self::with_path)
  }

  pub fn with_path(p: impl Into<PathBuf>) -> io::Result<Self> {
    let current_dir = p.into();
    let mut dir_content = Vec::<PathBuf>::new();
    for d in fs::read_dir(&current_dir)? {
      dir_content.push(d?.path().file_name().unwrap().into());
    }
    if current_dir.parent().is_some() {
      dir_content.push("..".into());
    }
    dir_content.sort();
    let mut content = LinearLayout::vertical();
    for p in &dir_content {
      content.add_child(FocusableTextView::new(TextView::new(p.to_string_lossy())));
    }
    let content = RcView::new(content);
    let err_msg = RcView::new(TextView::empty());
    let view = LinearLayout::vertical()
      .child(Panel::new(TextView::new(current_dir.to_string_lossy())))
      .child(content.clone().scrollable())
      .child(err_msg.clone());
    let mut ret = Self { current_dir, dir_content, last_click: None, view, content, err_msg };
    if !ret.dir_content.is_empty() {
      ret.recolor(0, 0);
    }
    Ok(ret)
  }

  pub fn focused_path(&self) -> (&PathBuf, &PathBuf) {
    (&self.current_dir, &self.dir_content[self.content.get().get_focus_index()])
  }

  fn recolor(&mut self, old: usize, new: usize) {
    let mut content = self.content.get();
    content.get_child_mut(old).unwrap().as_any_mut().downcast_mut::<FocusableTextView>().unwrap()
      .inner.set_content(self.dir_content[old].to_string_lossy());
    content.get_child_mut(new).unwrap().as_any_mut().downcast_mut::<FocusableTextView>().unwrap()
      .inner.set_content(StyledString::styled(self.dir_content[new].to_string_lossy(), Effect::Underline));
  }
}

impl ViewWrapper for PathSelector {
  cursive::wrap_impl!(self.view: LinearLayout);

  fn wrap_on_event(&mut self, event: Event) -> EventResult {
    let focus = self.content.get().get_focus_index();
    match event {
      Event::Key(Key::Up) => {
        if self.content.get().set_focus_index(focus.wrapping_sub(1)).is_ok() {
          self.recolor(focus, focus - 1);
        }
        EventResult::Consumed(None)
      }
      Event::Key(Key::Down) => {
        if self.content.get().set_focus_index(focus + 1).is_ok() {
          self.recolor(focus, focus + 1);
        }
        EventResult::Consumed(None)
      }
      _ => {
        let is_press = if let Event::Mouse { event: MouseEvent::Press(_), .. } = event { true } else { false };
        let ret = self.view.on_event(event);
        let new_focus = self.content.get().get_focus_index();
        self.recolor(focus, new_focus);
        if is_press {
          let update = if let Some(last_click) = self.last_click {
            if last_click.elapsed() < Duration::from_millis(500) {
              let p = if focus == 0 {
                self.current_dir.parent().unwrap().to_path_buf() // ".."
              } else {
                self.current_dir.join(&self.dir_content[focus])
              };
              match PathSelector::with_path(&p) {
                Ok(p) => *self = p,
                Err(e) => {
                  let msg = format!("failed to enter dir `{}`, reason: \"{}\"", p.display(), e);
                  self.err_msg.get().set_content(StyledString::styled(msg, Color::Dark(BaseColor::Red)));
                }
              }
              false
            } else { true }
          } else { true };
          self.last_click = if update { Some(Instant::now()) } else { None };
        }
        ret
      }
    }
  }
}

struct RcView<V> { inner: Rc<RefCell<V>> }

impl<V> RcView<V> {
  fn new(v: V) -> Self { Self { inner: Rc::new(RefCell::new(v)) } }

  fn get(&self) -> RefMut<V> { self.inner.borrow_mut() }

  fn clone(&self) -> Self { Self { inner: self.inner.clone() } }
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

struct FocusableTextView { inner: TextView }

impl FocusableTextView {
  fn new(t: TextView) -> Self { Self { inner: t } }
}

impl ViewWrapper for FocusableTextView {
  cursive::wrap_impl!(self.inner: TextView);

  fn wrap_take_focus(&mut self, _: direction::Direction) -> bool { true }
}