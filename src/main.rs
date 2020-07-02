use editor::*;
use druid::widget::{Checkbox, Flex, Label, MainAxisAlignment, Padding, Parse, Stepper, Switch, TextBox, WidgetExt, Scroll};
use druid::{AppLauncher, Data, Lens, LensExt, LensWrap, LocalizedString, Widget, WindowDesc};


pub mod languages;
pub mod editor;
pub mod data; pub use data::*;
pub mod spec; pub use spec::*;

fn build_widget() -> impl Widget<u64> {
    Scroll::new(
        EditorState::new()
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
