// SPDX-License-Identifier: MIT
// Copyright (c) 2025 HCLTech Ltd. All rights reserved.
// See the LICENSE file in the project root for more details.

mod config;
mod display;
mod platform;

use config::{DisplayBlock, load_config, run_command};
use display::Display;
use gpio_cdev::{Chip, LineRequestFlags};
use platform::get_i2c_bus;
use std::{sync::mpsc, thread, time::Duration, time::Instant};
use tracing::{debug, error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

enum RotaryEvent {
    Clockwise,
    CounterClockwise,
    ButtonPress,
}

fn update_display(
    display: &mut Display,
    block: &DisplayBlock,
) -> Result<(), Box<dyn std::error::Error>> {
    let top_line = block.get_top_line();
    let second_line = block.get_second_line();
    display.write_text(&top_line, &second_line)?;
    Ok(())
}

fn refresh_display(
    display: &mut Display,
    block: &DisplayBlock,
    last_refresh: &mut Instant,
    loaded_block_content: &mut bool,
) {
    if block.refresh_interval > 0
        && last_refresh.elapsed() >= Duration::from_secs(block.refresh_interval)
    {
        if let Err(e) = update_display(display, block) {
            error!("Error updating display: {}", e);
        }
        *last_refresh = Instant::now();
    } else if block.refresh_interval == 0 && !*loaded_block_content {
        *loaded_block_content = true;
        if let Err(e) = update_display(display, block) {
            error!("Error updating display: {}", e);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up file appender
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "/var/log/doggo-display",
        "doggo-display.log",
    );

    // Create a layer for file logging
    let file_layer = fmt::layer().with_writer(file_appender).with_ansi(false); // Disable ANSI colors in log files

    // Create a layer for stdout (for journalctl)
    let stdout_layer = fmt::layer().with_writer(std::io::stdout);

    // Register both layers with the subscriber
    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer)
        .with(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();

    info!("Starting up");

    info!("Loading config...");
    let config = load_config("config.toml").unwrap_or_else(|_| {
        error!("Failed to load config.toml");
        std::process::exit(1);
    });

    // Read I2C bus from config
    let i2c_bus = &config.hardware.i2c_bus;
    info!("Using I2C bus: {}", i2c_bus);

    // Initialize I2C using config
    let i2c = get_i2c_bus(i2c_bus)?;
    let mut display = Display::new(i2c);
    info!("Display initialized");

    if config.blocks.is_empty() {
        display.write_text("No config", "found")?;
        error!("No blocks found in config.toml");
        return Ok(());
    }
    info!("Config loaded");

    info!("Starting rotary encoder thread...");
    let gpio = config.hardware.clone();

    let (tx, rx) = mpsc::channel();

    thread::spawn({
        let tx = tx.clone();
        move || {
            if let Err(e) = rotary_encoder_thread(&gpio.chip, gpio.clk, gpio.dt, gpio.sw, &tx) {
                error!("Error in rotary encoder thread: {}", e);
            }
        }
    });
    info!("Rotary encoder thread started");

    let mut last_refresh = Instant::now();
    let mut loaded_block_content = false;
    let mut active_index = 0;

    loop {
        if let Ok(event) = rx.try_recv() {
            loaded_block_content = false;
            match event {
                RotaryEvent::Clockwise => {
                    debug!("Rotary event: Clockwise");
                    active_index = (active_index + 1) % config.blocks.len();
                }
                RotaryEvent::CounterClockwise => {
                    debug!("Rotary event: CounterClockwise");
                    if active_index == 0 {
                        active_index = config.blocks.len() - 1;
                    } else {
                        active_index -= 1;
                    }
                }
                RotaryEvent::ButtonPress => {
                    debug!("Rotary event: ButtonPress");
                    let block = &config.blocks[active_index];
                    if block.button_enabled {
                        let output = run_command(&block.function_to_run);
                        info!("Running function: {}", block.function_to_run);
                        display.write_text("Button Pressed", &output)?;
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
        }

        let block = &config.blocks[active_index];
        refresh_display(
            &mut display,
            block,
            &mut last_refresh,
            &mut loaded_block_content,
        );
        thread::sleep(Duration::from_millis(100));
    }
}

fn rotary_encoder_thread(
    chip_path: &str,
    clk_pin: u32,
    dt_pin: u32,
    sw_pin: u32,
    tx: &mpsc::Sender<RotaryEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut chip = Chip::new(chip_path)?;

    let clk = chip
        .get_line(clk_pin)?
        .request(LineRequestFlags::INPUT, 0, "clk")?;
    let dt = chip
        .get_line(dt_pin)?
        .request(LineRequestFlags::INPUT, 0, "dt")?;
    let sw = chip
        .get_line(sw_pin)?
        .request(LineRequestFlags::INPUT, 0, "sw")?;

    let mut last_clk = clk.get_value()?;
    let mut last_sw = sw.get_value()?;

    loop {
        let clk_value = clk.get_value()?;
        let dt_value = dt.get_value()?;
        let sw_value = sw.get_value()?;

        if clk_value != last_clk {
            if clk_value == 1 {
                if dt_value == 0 {
                    tx.send(RotaryEvent::Clockwise)?;
                } else {
                    tx.send(RotaryEvent::CounterClockwise)?;
                }
            }
            last_clk = clk_value;
        }

        if sw_value == 0 && last_sw == 1 {
            tx.send(RotaryEvent::ButtonPress)?;
        }
        last_sw = sw_value;

        thread::sleep(Duration::from_millis(10));
    }
}
