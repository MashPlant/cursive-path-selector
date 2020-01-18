use cursive::{views::*, traits::*, Cursive};
use cursive_path_selector::PathSelector;

fn main() {
  let mut s: Cursive = Cursive::default();

  s.add_layer(Dialog::around(PathSelector::new().unwrap().with_name("selector").max_width(40))
    .button("Ok", |s| {
      let selector = s.find_name::<PathSelector>("selector").unwrap();
      let (parent, last) = selector.focused_path();
      s.add_layer(Dialog::info(format!("You are focusing on \"{}\"", parent.join(last).display())));
    })
    .button("Quit", |s| s.quit()));

  s.run();
}