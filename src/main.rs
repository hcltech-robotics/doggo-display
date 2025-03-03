mod config;
mod display;
mod platform;

use config::{DisplayBlock, load_config};
use display::Display;
use platform::get_i2c_bus;
use std::{thread, time::Duration, time::Instant};

/// Updates the display with the top and second lines from the given block.
fn update_display(
    display: &mut Display,
    block: &DisplayBlock,
) -> Result<(), Box<dyn std::error::Error>> {
    let top_line = block.get_top_line();
    let second_line = block.get_second_line();
    display.write_text(&top_line, &second_line)?;
    Ok(())
}

/// Updates the display when it's time based on the refresh interval.
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
            eprintln!("Error updating display: {}", e);
        }
        *last_refresh = Instant::now();
    } else if block.refresh_interval == 0 && !*loaded_block_content {
        *loaded_block_content = true;
        if let Err(e) = update_display(display, block) {
            eprintln!("Error updating display: {}", e);
        }
    }
}

// This will hold the index of the active display block
static mut ACTIVE_BLOCK_INDEX: usize = 0;

fn get_active_block_index() -> usize {
    // Use the global ACTIVE_BLOCK_INDEX to return the current index
    unsafe { ACTIVE_BLOCK_INDEX }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up I2C and display
    let i2c = get_i2c_bus()?;
    let mut display = Display::new(i2c);

    // Load the configuration
    let config = load_config("config.toml").unwrap_or_else(|_| {
        display.write_text("Failed to", "load config.toml").unwrap();
        std::process::exit(1);
    });

    if config.blocks.is_empty() {
        display.write_text("No config", "found")?;
        return Ok(());
    }

    // Timing variables for display refresh
    let mut last_refresh = Instant::now();
    let mut loaded_block_content = false;

    loop {
        let active_index = get_active_block_index();
        let block = &config.blocks[active_index];

        // Refresh the display as needed
        refresh_display(
            &mut display,
            block,
            &mut last_refresh,
            &mut loaded_block_content,
        );

        thread::sleep(Duration::from_millis(100)); // Avoid busy-waiting
    }
}
