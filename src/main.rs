#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
use alloc::sync::Arc;

use embedded_term::TextOnGraphic;
use nx::console::vty::PersistentBufferedCanvas;
use nx::diag::abort;
use nx::gpu;
use nx::input;
use nx::result::*;
use nx::service::hid;
use nx::svc;
use nx::sync::RwLock;
use nx::thread;
use nx::util;

use core::fmt::Write;
use core::panic;
use core::time::Duration;

// nx::rrt0_define_module_name!("console-interactive");

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    let _info_message = format!("{}", info);
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
    let mut console: nx::console::vty::TextBufferConsole = {
        let gpu_ctx = gpu::Context::new(
            gpu::NvDrvServiceKind::Application,
            gpu::ViServiceKind::System,
            0x40000,
        )
        .unwrap();

        let surface = nx::gpu::canvas::CanvasManager::new_stray(
            Arc::new(RwLock::new(gpu_ctx)),
            Default::default(),
            2,
            gpu::BlockLinearHeights::FourGobs,
        )
        .unwrap();

        let width = surface.surface.width();
        let height = surface.surface.height();

        let text_buffer = TextOnGraphic::new(PersistentBufferedCanvas::new(surface), width, height);

        embedded_term::Console::on_text_buffer(text_buffer)
    };

    for _ in 0..10 {
        let _ = console.write_str("fuckkkkkkkkkkk");
        let _ = thread::sleep(Duration::from_secs(1).as_nanos() as i64);
    }
}
