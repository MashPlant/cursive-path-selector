mod util;

use std::{path::*, time::*, io, env, fs};
use cursive::{view::*, views::*, event::*, theme::*, utils::markup::StyledString};
use util::*;

/// A cursive view that can display directories and allow user to select a path in the file system.
///
/// Double click to enter a sub directory.
pub struct PathSelector {
  current_dir: PathBuf,
  dir_content: Vec<PathBuf>,
  last_click: Option<Instant>,
  view: LinearLayout,
  // below are reference to the views inside `view`
  content: RcView<NonAutoScrollView<LinearLayout>>,
  err_msg: RcView<TextView>,
}

impl PathSelector {
  /// Return a `PathSelector` instance with the current working directory on success.
  ///
  /// Return `Err(io::Error)` if an error occurs when reading the directory or when reading contents from it.
  pub fn new() -> io::Result<Self> {
    env::current_dir().and_then(Self::with_path)
  }

  /// Like `PathSelector::new`, but open the directory that `p` specified instead of the current working directory.
  pub fn with_path(p: impl Into<PathBuf>) -> io::Result<Self> {
    let current_dir = p.into();
    let mut dir_content = Vec::<PathBuf>::new();
    for d in fs::read_dir(&current_dir)? {
      dir_content.push(d?.path().file_name().unwrap().into());
    }
    if current_dir.parent().is_some() {
      // there is no cross-platform issue, even if there is a weird platform that doesn't use ".." to represent parent directory
      // because we won't use ".." directly, it is just a symbol for the user of the tui program
      dir_content.push("..".into());
    }
    dir_content.sort_unstable(); // sort it, because the order of `read_dir` is not specified
    let mut content = LinearLayout::vertical();
    for p in &dir_content {
      content.add_child(FocusableTextView::new(TextView::new(p.to_string_lossy())));
    }
    let content = RcView::new(NonAutoScrollView::new(content));
    let err_msg = RcView::new(TextView::empty());
    let view = LinearLayout::vertical()
      .child(Panel::new(TextView::new(current_dir.to_string_lossy())))
      .child(content.clone())
      .child(err_msg.clone());
    let mut ret = Self { current_dir, dir_content, last_click: None, view, content, err_msg };
    if !ret.dir_content.is_empty() {
      ret.recolor(0, 0);
    }
    Ok(ret)
  }

  /// Return (current opening directory, current focused sub path).
  ///
  /// User can use `Path::join` to join these two paths to get a full path.
  ///
  /// The initially focused sub path is "..".
  pub fn focused_path(&self) -> (&PathBuf, &PathBuf) {
    (&self.current_dir, &self.dir_content[self.content.get().inner().get_focus_index()])
  }

  // clear the style on the last selected path, and add style to the new one
  fn recolor(&mut self, old: usize, new: usize) {
    let mut content = self.content.get();
    content.inner().get_child_mut(old).unwrap().as_any_mut().downcast_mut::<FocusableTextView>().unwrap()
      .get().set_content(self.dir_content[old].to_string_lossy());
    content.inner().get_child_mut(new).unwrap().as_any_mut().downcast_mut::<FocusableTextView>().unwrap()
      .get().set_content(StyledString::styled(self.dir_content[new].to_string_lossy(), Effect::Underline));
  }
}

impl ViewWrapper for PathSelector {
  cursive::wrap_impl!(self.view: LinearLayout);

  fn wrap_on_event(&mut self, event: Event) -> EventResult {
    let focus = self.content.get().inner().get_focus_index();
    match event {
      Event::Key(Key::Up) => {
        if self.content.get().inner().set_focus_index(focus.wrapping_sub(1)).is_ok() {
          self.recolor(focus, focus - 1);
        }
        EventResult::Consumed(None)
      }
      Event::Key(Key::Down) => {
        // I guess it is not possible to have integer overflow...
        if self.content.get().inner().set_focus_index(focus + 1).is_ok() {
          self.recolor(focus, focus + 1);
        }
        EventResult::Consumed(None)
      }
      _ => {
        let is_press = if let Event::Mouse { event: MouseEvent::Press(_), .. } = event { true } else { false };
        let ret = self.view.on_event(event);
        let new_focus = self.content.get().inner().get_focus_index();
        self.recolor(focus, new_focus);
        if is_press {
          let update = if let Some(last_click) = self.last_click {
            if last_click.elapsed() < Duration::from_millis(500) {
              let p = if let (Some(p), true) = (self.current_dir.parent(), focus == 0) {
                // focus == 0 && self.current_dir.parent().is_some() means the selected path is ".."
                // so go back to parent directory
                p.to_path_buf()
              } else {
                self.current_dir.join(&self.dir_content[focus])
              };
              match PathSelector::with_path(&p) {
                Ok(p) => *self = p, // here the previous err msg can be cleared
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