use macroquad::prelude::*;
use std::sync::Once;

#[cfg(feature = "profiling")]
use profiling;

#[cfg(feature = "profiling")]
static INIT: Once = Once::new();

pub struct TracingProfiler;

impl TracingProfiler {
    pub fn new() -> Self {
        #[cfg(feature = "profiling")]
        {
            INIT.call_once(|| {
                info!("Initializing tracing profiler");
                Self::init_tracing();

                #[cfg(all(target_arch = "wasm32", feature = "profiling-wasm"))]
                {
                    info!("🔍 Tracing profiler enabled with tracing-wasm (Chrome Performance tab)");
                    println!(
                        "🔍 Tracing profiler enabled with tracing-wasm (Chrome Performance tab)"
                    );
                }

                #[cfg(all(target_arch = "wasm32", not(feature = "profiling-wasm")))]
                {
                    info!("🔍 Tracing profiler enabled with console output");
                    println!("🔍 Tracing profiler enabled with console output");
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    info!("🔍 Tracing profiler enabled with hierarchical output");
                }
            });
        }

        #[cfg(not(feature = "profiling"))]
        {
            info!("⚪ Tracing profiler disabled (profiling feature not enabled)");
            #[cfg(target_arch = "wasm32")]
            println!("⚪ Tracing profiler disabled (profiling feature not enabled)");
        }

        Self
    }

    #[cfg(all(feature = "profiling", not(target_arch = "wasm32")))]
    fn init_tracing() {
        use tracing_subscriber::prelude::*;
        use tracing_tree::HierarchicalLayer;

        tracing_subscriber::registry()
            .with(
                HierarchicalLayer::new(2)
                    .with_targets(true)
                    .with_bracketed_fields(true)
                    .with_thread_names(false)
                    .with_thread_ids(false)
                    .with_timer(tracing_tree::time::Uptime::default())
                    .with_ansi(true),
            )
            .init();
    }

    #[cfg(all(feature = "profiling", target_arch = "wasm32"))]
    fn init_tracing() {
        // For WASM, try to use tracing-wasm if available, otherwise fallback to basic console
        #[cfg(feature = "profiling-wasm")]
        {
            info!("🔍 Tracing profiler enabled with tracing-wasm");
            tracing_wasm::set_as_global_default();
        }

        #[cfg(not(feature = "profiling-wasm"))]
        {
            use tracing_subscriber::prelude::*;

            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_timer(tracing_subscriber::fmt::time::uptime())
                        .with_target(true)
                        .with_level(true)
                        .with_ansi(false), // No ANSI colors in browser console
                )
                .init();
        }
    }

    pub fn new_frame(&mut self) {
        #[cfg(feature = "profiling")]
        profiling::finish_frame!();
    }

    pub fn handle_input(&mut self) {
        // No-op - input handling is now done via tracing spans
    }

    pub fn render_ui(&mut self) {
        // No-op - UI rendering is now done via tracing spans
    }

    pub fn log_frame_stats(&mut self) {
        // No-op - tracing handles all output automatically
        // We could add periodic summary stats here if needed
    }
}

// Re-export tracing macros for convenience
#[cfg(feature = "profiling")]
pub use tracing::{debug_span, info_span, instrument, trace_span};

// Provide no-op macros when profiling is disabled
#[cfg(not(feature = "profiling"))]
pub mod disabled {
    macro_rules! info_span {
        ($($args:tt)*) => {{
            // Return a dummy guard that does nothing
            DisabledSpanGuard
        }};
    }

    macro_rules! debug_span {
        ($($args:tt)*) => {{
            DisabledSpanGuard
        }};
    }

    macro_rules! trace_span {
        ($($args:tt)*) => {{
            DisabledSpanGuard
        }};
    }

    pub use {debug_span, info_span, trace_span};

    pub struct DisabledSpanGuard;

    impl DisabledSpanGuard {
        pub fn entered(self) -> Self {
            self
        }
    }

    impl Drop for DisabledSpanGuard {
        fn drop(&mut self) {}
    }
}

#[cfg(not(feature = "profiling"))]
pub use disabled::*;
