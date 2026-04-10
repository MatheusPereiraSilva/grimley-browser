use tao::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub fn create_window(event_loop: &EventLoop<()>) -> Window {
    WindowBuilder::new()
        .with_title("Grimley Browser 🧠")
        .with_inner_size(LogicalSize::new(1000.0, 800.0))
        .build(event_loop)
        .expect("Erro ao criar janela")
}
