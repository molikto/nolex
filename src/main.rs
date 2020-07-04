use editor::*;
use druid::widget::{Scroll};
use druid::{AppLauncher, LocalizedString, Widget, WindowDesc};


pub mod languages;
pub mod editor;
pub mod data; pub use data::*;
pub mod spec; pub use spec::*;
pub mod language; pub use language::*;

fn build_widget() -> impl Widget<u64> {
    Scroll::new(
        EditorWidget::new()
    ).vertical()
}

pub fn main() {
    // TODO use WINDOW_BACKGROUND_COLOR as our bg color
    let window = WindowDesc::new(build_widget)
        .title(LocalizedString::new("window-title").with_placeholder("nolex"));
    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(0)
        .expect("launch failed");
}
