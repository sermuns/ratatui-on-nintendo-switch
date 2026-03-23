#![no_std]
#![no_main]

extern crate alloc;
use alloc::{string::String, sync::Arc, vec};

use core::panic;
use mousefood::prelude::*;
use nx::{diag::abort, gpu, input, result::ResultBase, service::hid, svc, sync::RwLock, util};
use ratatui::{
    Frame, Terminal,
    text::Line,
    widgets::{Block, Paragraph},
};

// neccessary?
nx::rrt0_define_module_name!(env!("CARGO_PKG_NAME"));

#[panic_handler]
fn panic_handler(_info: &panic::PanicInfo) -> ! {
    nx::diag::abort::abort(abort::AbortLevel::Panic(), nx::rc::ResultPanicked::make());
}

#[unsafe(no_mangle)]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    } else {
        let heap_size: usize = 0x10000000;
        let heap_address = svc::set_heap_size(heap_size).unwrap();
        util::PointerAndSize::new(heap_address, heap_size)
    }
}

#[unsafe(no_mangle)]
fn main() {
    let mut canvas = {
        let gpu_ctx = gpu::Context::new(
            gpu::NvDrvServiceKind::Applet,
            gpu::ViServiceKind::System,
            0x800000,
        )
        .unwrap();

        let surface = nx::gpu::canvas::CanvasManager::new_stray(
            Arc::new(RwLock::new(gpu_ctx)),
            Default::default(),
            2,
            gpu::BlockLinearHeights::EightGobs,
        )
        .unwrap();

        nx::console::vty::PersistentBufferedCanvas::new(surface)
    };

    let backend = EmbeddedBackend::new(
        &mut canvas,
        EmbeddedBackendConfig {
            font_regular: embedded_graphics_unicodefonts::MONO_10X20,
            ..Default::default()
        },
    );

    let mut terminal = Terminal::new(backend).unwrap();

    // allow ALL controller types!
    // this sets the unused bits 31 and 32 too,
    // will that be a problem..?
    let supported_style_tags = !hid::NpadStyleTag::default();
    const CONTROLLERS_TO_POLL: [hid::NpadIdType; 2] =
        [hid::NpadIdType::Handheld, hid::NpadIdType::No1];
    let input_ctx = input::Context::new(supported_style_tags, CONTROLLERS_TO_POLL.len())
        .expect("failed to create input context");
    let mut players = CONTROLLERS_TO_POLL.map(|controller| input_ctx.get_player(controller));

    let mut message = String::new();

    'main: loop {
        for player in &mut players {
            let buttons_down = player.get_buttons_down();
            if buttons_down.contains(hid::NpadButton::Left()) {
                message += "Left";
            } else if buttons_down.contains(hid::NpadButton::Right()) {
                message += "Right";
            } else if buttons_down.contains(hid::NpadButton::Up()) {
                message += "Up";
            } else if buttons_down.contains(hid::NpadButton::Down()) {
                message += "Down";
            } else if buttons_down.contains(hid::NpadButton::B()) {
                message.clear();
            } else if buttons_down.contains(hid::NpadButton::Plus()) {
                break 'main;
            }
        }
        let _ = terminal.draw(|f| render(f, &message));
    }
}

fn render(frame: &mut Frame, message: &str) {
    // TODO: avoid Vec
    let lines = vec![Line::from("Hello from Mousefood!"), message.into()];
    let paragraph = Paragraph::new(lines).block(Block::bordered().title(env!("CARGO_PKG_NAME")));
    frame.render_widget(paragraph, frame.area());
}
