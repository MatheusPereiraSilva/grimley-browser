use tao::window::Window;
use wry::{
    dpi::{LogicalPosition, LogicalSize, Position, Size},
    Rect,
};

const TOOLBAR_HEIGHT: f64 = 96.0;

pub fn toolbar_bounds(window: &Window) -> Rect {
    let size = window.inner_size().to_logical::<f64>(window.scale_factor());

    Rect {
        position: Position::Logical(LogicalPosition::new(0.0, 0.0)),
        size: Size::Logical(LogicalSize::new(size.width, TOOLBAR_HEIGHT)),
    }
}

pub fn content_bounds(window: &Window) -> Rect {
    let size = window.inner_size().to_logical::<f64>(window.scale_factor());
    let content_height = (size.height - TOOLBAR_HEIGHT).max(0.0);

    Rect {
        position: Position::Logical(LogicalPosition::new(0.0, TOOLBAR_HEIGHT)),
        size: Size::Logical(LogicalSize::new(size.width, content_height)),
    }
}
