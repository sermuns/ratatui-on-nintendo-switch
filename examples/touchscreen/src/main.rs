#![no_std]
#![no_main]

extern crate alloc;
use alloc::{
    format,
    sync::Arc,
    vec::{self, Vec},
};

use core::{panic, time::Duration};
use mousefood::prelude::*;
use nx::{
    diag::abort,
    gpu, input,
    result::*,
    service::{
        self, hid,
        psm::{IPsmClient, PsmService},
    },
    svc,
    sync::RwLock,
    thread::sleep,
    util,
};
use ratatui::{
    layout::Rows,
    widgets::{Block, Cell, Padding, Paragraph, Row, Table},
};
use ratatui::{prelude::*, widgets::Gauge};

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
    // // enable screenshotting??
    // if let Some(applet_proxy) = applet::get_applet_proxy().as_ref()
    //     && let Ok(self_controller) = applet_proxy.get_self_controller()
    // {
    //     let _ = self_controller.set_screenshot_permission(ScreenShotPermission::Enable);
    // }

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

    let app = App::new(&input_ctx);

    'main: loop {
        let _ = terminal.draw(|f| app.render(f));

        for player in &mut players {
            let buttons_down = player.get_buttons_down();
            if buttons_down.intersects(hid::NpadButton::Plus() | hid::NpadButton::B()) {
                break 'main;
            }
        }

        const REFRESH_INTERVAL: i64 = Duration::from_millis(100).as_nanos() as i64;
        let _ = sleep(REFRESH_INTERVAL);
    }
}

struct App<'a> {
    input_ctx: &'a input::Context,
}

impl<'a> App<'a> {
    fn new(input_ctx: &'a input::Context) -> Self {
        Self { input_ctx }
    }

    fn render(&self, frame: &mut Frame) {
        // let layout = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]);
        // let [top_area, bottom_area] = frame.area().layout(&layout);

        // let lines: Vec<_> = self
        //     .input_ctx
        //     .get_touch_state()
        //     .touches
        //     .iter()
        //     .map(|touch| {
        //         format!(
        //             "x: {} | y: {} | diameter_x: {} | diameter_y: {} | rotation_angle: {}",
        //             touch.x, touch.y, touch.diameter_x, touch.diameter_y, touch.rotation_angle as i16
        //         )
        //         .into()
        //     })
        //     .collect();
        // let dbg_paragraph = Paragraph::new(lines);

        let rows = self.input_ctx.get_touch_state().touches.map(|touch| {
            Row::new([
                Cell::new(format!("x: {}", touch.x)),
                Cell::new(format!("y: {}", touch.y)),
                Cell::new(format!("diameter_x: {}", touch.diameter_x)),
                Cell::new(format!("diameter_y: {}", touch.diameter_y)),
                Cell::new(format!("rotation_angle: {}", touch.rotation_angle as i16)),
            ])
        });

        let widths = [Constraint::Fill(1); 5];

        let table = Table::new(rows, widths).block(Block::new().padding(Padding::horizontal(5)));

        frame.render_widget(
            table,
            frame.area().centered_vertically(Constraint::Ratio(1, 2)),
        );
    }
}
