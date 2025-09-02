use macroquad::prelude::*;
use std::collections::HashMap;

pub struct Profiler {
    #[cfg(feature = "profiling")]
    scope_times: HashMap<&'static str, ScopeTimer>,
}

#[cfg(feature = "profiling")]
struct ScopeTimer {
    total_time: f64,
    sample_count: u32,
    max_time: f64,
}

#[cfg(feature = "profiling")]
impl ScopeTimer {
    fn new() -> Self {
        Self {
            total_time: 0.0,
            sample_count: 0,
            max_time: 0.0,
        }
    }

    fn add_sample(&mut self, time: f64) {
        self.total_time += time;
        self.sample_count += 1;
        if time > self.max_time {
            self.max_time = time;
        }
    }

    fn get_average(&self) -> f64 {
        if self.sample_count > 0 {
            self.total_time / self.sample_count as f64
        } else {
            0.0
        }
    }

    fn reset(&mut self) {
        self.total_time = 0.0;
        self.sample_count = 0;
        self.max_time = 0.0;
    }
}

impl Profiler {
    pub fn new() -> Self {
        #[cfg(feature = "profiling")]
        {
            info!("🔍 Console profiler enabled");
            #[cfg(target_arch = "wasm32")]
            println!("🔍 Console profiler enabled");
        }

        #[cfg(not(feature = "profiling"))]
        {
            info!("⚪ Profiler disabled (profiling feature not enabled)");
            #[cfg(target_arch = "wasm32")]
            println!("⚪ Profiler disabled (profiling feature not enabled)");
        }

        Self {
            #[cfg(feature = "profiling")]
            scope_times: HashMap::new(),
        }
    }

    pub fn new_frame(&mut self) {
        #[cfg(feature = "profiling")]
        profiling::finish_frame!();
    }

    pub fn handle_input(&mut self) {
        // No input handling needed for console profiler
    }

    pub fn render_ui(&mut self) {
        // No UI rendering needed for console profiler
    }

    #[cfg(feature = "profiling")]
    pub fn record_scope_time(&mut self, scope_name: &'static str, time_ms: f64) {
        let timer = self.scope_times.entry(scope_name).or_insert_with(ScopeTimer::new);
        timer.add_sample(time_ms);
    }

    #[cfg(feature = "profiling")]
    pub fn start_scope(&self, _scope_name: &'static str) -> f64 {
        get_time()
    }

    #[cfg(feature = "profiling")]
    pub fn end_scope(&mut self, scope_name: &'static str, start_time: f64) {
        let end_time = get_time();
        let duration_ms = (end_time - start_time) * 1000.0;
        self.record_scope_time(scope_name, duration_ms);
    }

    #[cfg(not(feature = "profiling"))]
    pub fn start_scope(&self, _scope_name: &'static str) -> f64 {
        0.0
    }

    #[cfg(not(feature = "profiling"))]
    pub fn end_scope(&mut self, _scope_name: &'static str, _start_time: f64) {
        // No-op
    }

    pub fn log_frame_stats(&mut self) {
        #[cfg(feature = "profiling")]
        {
            // Log frame statistics to console periodically
            static mut FRAME_COUNT: u32 = 0;
            unsafe {
                FRAME_COUNT += 1;
                if FRAME_COUNT % 10 == 0 {
                    // Simple FPS logging using get_time()
                    static mut LAST_LOG_TIME: f64 = 0.0;
                    let current_time = get_time();
                    if LAST_LOG_TIME == 0.0 {
                        LAST_LOG_TIME = current_time;
                        return; // Skip first measurement
                    }

                    let time_diff = current_time - LAST_LOG_TIME;
                    if time_diff > 0.0 {
                        let fps = 10.0 / time_diff; // 10 frames over time_diff seconds
                        let avg_frame_time = time_diff * 1000.0 / 10.0; // ms per frame

                        let profile_msg =
                            format!("Avg frame time: {:.2}ms, FPS: {:.1}", avg_frame_time, fps);
                        info!("🔍 Profiler: {}", profile_msg);

                        // Log detailed scope timing
                        self.log_scope_details();

                        // For WASM, also log to console.log for browser dev tools
                        #[cfg(target_arch = "wasm32")]
                        {
                            println!("🔍 Profiler: {}", profile_msg);
                            self.log_scope_details_wasm();
                        }

                        // Reset scope timers for next measurement window
                        for timer in self.scope_times.values_mut() {
                            timer.reset();
                        }
                    }

                    LAST_LOG_TIME = current_time;
                }
            }
        }
    }

    #[cfg(feature = "profiling")]
    fn log_scope_details(&self) {
        if self.scope_times.is_empty() {
            return;
        }

        // Sort scopes by average time (descending)
        let mut sorted_scopes: Vec<_> = self.scope_times.iter().collect();
        sorted_scopes.sort_by(|a, b| b.1.get_average().partial_cmp(&a.1.get_average()).unwrap_or(std::cmp::Ordering::Equal));

        info!("📊 Scope Timing (Top 10):");
        for (scope_name, timer) in sorted_scopes.iter().take(10) {
            if timer.sample_count > 0 {
                info!(
                    "  {} avg:{:.2}ms max:{:.2}ms samples:{}",
                    scope_name,
                    timer.get_average(),
                    timer.max_time,
                    timer.sample_count
                );
            }
        }
    }

    #[cfg(all(feature = "profiling", target_arch = "wasm32"))]
    fn log_scope_details_wasm(&self) {
        if self.scope_times.is_empty() {
            return;
        }

        // Sort scopes by average time (descending)
        let mut sorted_scopes: Vec<_> = self.scope_times.iter().collect();
        sorted_scopes.sort_by(|a, b| b.1.get_average().partial_cmp(&a.1.get_average()).unwrap_or(std::cmp::Ordering::Equal));

        println!("📊 Scope Timing (Top 10):");
        for (scope_name, timer) in sorted_scopes.iter().take(10) {
            if timer.sample_count > 0 {
                println!(
                    "  {} avg:{:.2}ms max:{:.2}ms samples:{}",
                    scope_name,
                    timer.get_average(),
                    timer.max_time,
                    timer.sample_count
                );
            }
        }
    }
}