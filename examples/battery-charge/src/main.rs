#![no_std]
#![no_main]

extern crate alloc;
use alloc::{format, sync::Arc, vec};

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
use ratatui::widgets::{Block, Paragraph};
use ratatui::{prelude::*, widgets::Gauge};
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

    let psm = service::new_service_object().unwrap();

    let app = App::new(psm);

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

struct App {
    psm_service: PsmService,
}

impl App {
    fn new(psm_service: PsmService) -> Self {
        Self { psm_service }
    }

    fn render(&self, frame: &mut Frame) {
        // TODO: do less alloc...
        let percentage = match self.psm_service.get_battery_charge_percentage() {
            Err(e) => {
                let paragraph = Paragraph::new(format!("error: {e:?}"))
                    .block(Block::bordered().title(env!("CARGO_PKG_NAME")));
                frame.render_widget(paragraph, frame.area());
                return;
            }
            Ok(percentage) => percentage,
        };

        let layout = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [text_area, gauge_area, footer_area] = frame.area().layout(&layout);

        let big_text = BigText::builder()
            .centered()
            .pixel_size(PixelSize::Full)
            .style(Style::new().blue())
            .lines(vec![format!("{percentage}%").yellow().into()])
            .build();
        frame.render_widget(
            big_text,
            text_area.centered_vertically(Constraint::Percentage(50)),
        );

        let gauge_style = match percentage {
            0..=20 => Style::new().green().on_black(),
            21..=50 => Style::new().yellow().on_black(),
            _ => Style::new().white().green().on_black(),
        };
        let gauge = Gauge::default()
            .gauge_style(gauge_style)
            .percent(percentage as u16)
            .block(Block::new().title("Battery charge"));

        frame.render_widget(gauge, gauge_area);

        let paragraph = Paragraph::new("Press + or B to exit");
        frame.render_widget(paragraph, footer_area);
    }
}
