mod config;
mod display;
mod platform;

use config::{DisplayBlock, load_config, run_command};
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
    sw_line: u32,
    clk_state: &mut u8,
    dt_state: &mut u8,
    sw_state: &mut u8,
) -> Option<i32> {
    // Initialize GPIO chip (should be done once at startup)
    let mut chip = Chip::new("/dev/gpiochip4").unwrap();

    // Request lines for CLK, DT, and SW (do this once in your setup, not in every loop)
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
    let sw = chip
        .get_line(sw_line)
        .unwrap()
        .request(LineRequestFlags::INPUT, 0, "sw")
        .unwrap();

    // Read current state of CLK, DT, and SW
    let clk_value = clk.get_value().unwrap();
    let dt_value = dt.get_value().unwrap();
    let sw_value = sw.get_value().unwrap();

    // Debounce: check if state change has occurred for CLK and DT
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

    // Detect button press (SW state change)
    if sw_value == 0 && *sw_state == 1 {
        *sw_state = 0; // Button is pressed
        return Some(42); // Return a special value to indicate button press
    } else if sw_value == 1 && *sw_state == 0 {
        *sw_state = 1; // Button is released
    }

    // If no rotation or button press detected, return None
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
    let mut button_pressed = false;

    // Set GPIO pins for rotary encoder
    let clk_pin = 17;
    let dt_pin = 27;
    let sw_pin = 22;

    // Initialize state variables for CLK and DT
    let mut clk_state = 1; // Initial state (high)
    let mut dt_state = 1; // Initial state (high)
    let mut sw_state = 1; // Initial state (button not pressed)

    loop {
        // Read rotation direction from the rotary encoder
        if let Some(rotation) = read_rotation(
            clk_pin,
            dt_pin,
            sw_pin,
            &mut clk_state,
            &mut dt_state,
            &mut sw_state,
        ) {
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
                42 => {
                    button_pressed = true;
                    active_index
                } // Button press
                _ => active_index,
            };
            set_active_block_index(new_index);
        }

        let active_index = get_active_block_index();
        let block: &DisplayBlock = &config.blocks[active_index];

        if button_pressed & block.button_enabled {
            let function = &block.function_to_run;
            println!("Running function: {}", function);
            let output = run_command(function);
            println!("Output: {}", output);
            display.write_text("Button Pressed", &output)?;
            button_pressed = false;
            thread::sleep(Duration::from_secs(2));
        }

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
