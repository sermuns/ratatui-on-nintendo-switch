#![no_std]
#![no_main]

extern crate alloc;
use alloc::sync::Arc;
use core::{panic, time::Duration};
use mousefood::prelude::*;
use mousefood_logo_widget::MouseFoodLogo;
use nx::{
    diag::abort, gpu, input, result::ResultBase, service::hid, svc, sync::RwLock, thread::sleep,
    util,
};
use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::widgets::RatatuiLogo;
use tui_big_text::{BigText, PixelSize};

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
            gpu::BlockLinearHeights::TwoGobs,
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

    'main: loop {
        let _ = terminal.draw(render);

        for player in &mut players {
            let buttons_down = player.get_buttons_down();
            if buttons_down.intersects(hid::NpadButton::Plus() | hid::NpadButton::B()) {
                break 'main;
            }
        }
    }
}

fn render(frame: &mut Frame) {
    const POWERED_TEXT_HEIGHT_CONSTRAINT: Constraint = Constraint::Length(4);
    const MOUSEFOOD_LOGO_HEIGHT_CONSTRAINT: Constraint = Constraint::Length(6);
    let vertical_layout = Layout::vertical([
        POWERED_TEXT_HEIGHT_CONSTRAINT,
        MOUSEFOOD_LOGO_HEIGHT_CONSTRAINT,
    ])
    .spacing(1)
    .flex(Flex::Center);
    let [top_area, bottom_area] = frame.area().layout(&vertical_layout);

    let powered_text = BigText::builder()
        .pixel_size(PixelSize::Quadrant)
        .lines(["Powered by".into()])
        .centered()
        .build();
    frame.render_widget(powered_text, top_area);

    const RATATUI_LOGO_WIDTH_CONSTRAINT: Constraint = Constraint::Length(15);
    const AND_TEXT: &str = "and";
    const AND_TEXT_WIDTH_CONSTRAINT: Constraint = Constraint::Length(AND_TEXT.len() as u16);
    const MOUSEFOOD_LOGO_WIDTH_CONSTRAINT: Constraint = Constraint::Length(2 * 20);

    let horizontal_layout = Layout::horizontal([
        RATATUI_LOGO_WIDTH_CONSTRAINT,
        AND_TEXT_WIDTH_CONSTRAINT,
        MOUSEFOOD_LOGO_WIDTH_CONSTRAINT,
    ])
    .spacing(2)
    .flex(Flex::Center);
    let [ratatui_logo_area, and_area, mousefood_logo_area] = bottom_area.layout(&horizontal_layout);

    frame.render_widget(
        RatatuiLogo::default(),
        ratatui_logo_area.centered(RATATUI_LOGO_WIDTH_CONSTRAINT, Constraint::Length(2)),
    );

    frame.render_widget(
        AND_TEXT,
        and_area.centered(AND_TEXT_WIDTH_CONSTRAINT, Constraint::Length(1)),
    );

    frame.render_widget(
        MouseFoodLogo::default(),
        mousefood_logo_area.centered(
            MOUSEFOOD_LOGO_WIDTH_CONSTRAINT,
            MOUSEFOOD_LOGO_HEIGHT_CONSTRAINT,
        ),
    );
}
