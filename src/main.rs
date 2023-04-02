use druid::widget::Container;
use druid::{WindowDesc, AppLauncher, Widget};
use widget::{Content, AppState};

mod engine;
mod widget;

fn main() {
    let win = WindowDesc::new(build_ui_widget())
        .title("TicTacToe")
        .window_size((400.0, 400.0));

    // start the application
    AppLauncher::with_window(win)
        .launch(AppState::default())
        .expect("Failed to launch application");
}
fn build_ui_widget() -> impl Widget<AppState> {
    Container::new(Content::default())
}