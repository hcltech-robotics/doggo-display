mod config;
mod display;
mod platform;

use config::{DisplayBlock, load_config};
use display::Display;
use gpio_cdev::{Chip, LineRequestFlags};
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

/// Reads the current state of the rotary encoder and returns the direction of rotation.
/// Also implements debouncing and rotation detection improvements.
fn read_rotation(
    clk_line: u32,
    dt_line: u32,
    clk_state: &mut u8,
    dt_state: &mut u8,
) -> Option<i32> {
    // Initialize GPIO chip (should be done once at startup)
    let mut chip = Chip::new("/dev/gpiochip4").unwrap();

    // Request lines for CLK and DT (do this once in your setup, not in every loop)
    let clk = chip
        .get_line(clk_line)
        .unwrap()
        .request(LineRequestFlags::INPUT, 0, "clk")
        .unwrap();
    let dt = chip
        .get_line(dt_line)
        .unwrap()
        .request(LineRequestFlags::INPUT, 0, "dt")
        .unwrap();

    // Read current state of CLK and DT
    let clk_value = clk.get_value().unwrap();
    let dt_value = dt.get_value().unwrap();

    // Debounce: check if state change has occurred (only process change if state has changed)
    if clk_value != *clk_state {
        *clk_state = clk_value;

        // Determine rotation direction based on the state of DT
        if clk_value == 0 {
            if dt_value == 1 {
                return Some(1); // Clockwise
            } else {
                return Some(-1); // Counter-clockwise
            }
        }
    }

    // If no rotation detected, return None
    None
}

// This will hold the index of the active display block
static mut ACTIVE_BLOCK_INDEX: usize = 0;

fn get_active_block_index() -> usize {
    // Use the global ACTIVE_BLOCK_INDEX to return the current index
    unsafe { ACTIVE_BLOCK_INDEX }
}

fn set_active_block_index(index: usize) {
    // Update the active block index globally
    unsafe {
        ACTIVE_BLOCK_INDEX = index;
    }
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

    // Set GPIO pins for rotary encoder
    let clk_pin = 17;
    let dt_pin = 27;

    // Initialize state variables for CLK and DT
    let mut clk_state = 0;
    let mut dt_state = 0;

    loop {
        // Read rotation direction from the rotary encoder
        if let Some(rotation) = read_rotation(clk_pin, dt_pin, &mut clk_state, &mut dt_state) {
            loaded_block_content = false; // Reset loaded content flag
            let active_index = get_active_block_index();
            let new_index = match rotation {
                1 => (active_index + 1) % config.blocks.len(), // Clockwise: Next block
                -1 => {
                    if active_index == 0 {
                        config.blocks.len() - 1
                    } else {
                        active_index - 1
                    }
                } // Counter-clockwise: Previous block
                _ => active_index,
            };
            set_active_block_index(new_index);
        }

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
