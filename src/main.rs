mod config;
mod display;
mod platform;

use config::{DisplayBlock, load_config, run_command};
use display::Display;
use gpio_cdev::{Chip, LineRequestFlags};
use platform::get_i2c_bus;
use std::{sync::mpsc, thread, time::Duration, time::Instant};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let i2c = get_i2c_bus()?;
    let mut display = Display::new(i2c);

    let config = load_config("config.toml").unwrap_or_else(|_| {
        display.write_text("Failed to", "load config.toml").unwrap();
        std::process::exit(1);
    });

    if config.blocks.is_empty() {
        display.write_text("No config", "found")?;
        return Ok(());
    }

    let gpio = config.gpio.clone();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || rotary_encoder_thread(&gpio.chip, gpio.clk, gpio.dt, gpio.sw, tx));

    let mut last_refresh = Instant::now();
    let mut loaded_block_content = false;
    let mut active_index = 0;

    loop {
        if let Ok(event) = rx.try_recv() {
            loaded_block_content = false;
            match event {
                RotaryEvent::Clockwise => {
                    active_index = (active_index + 1) % config.blocks.len();
                }
                RotaryEvent::CounterClockwise => {
                    if active_index == 0 {
                        active_index = config.blocks.len() - 1;
                    } else {
                        active_index -= 1;
                    }
                }
                RotaryEvent::ButtonPress => {
                    let block = &config.blocks[active_index];
                    if block.button_enabled {
                        let output = run_command(&block.function_to_run);
                        println!("Running function: {}", block.function_to_run);
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
    tx: mpsc::Sender<RotaryEvent>,
) {
    let mut chip = Chip::new(chip_path).expect("Failed to open GPIO chip");

    let clk = chip
        .get_line(clk_pin)
        .expect("Failed to get CLK line")
        .request(LineRequestFlags::INPUT, 0, "clk")
        .expect("Failed to request CLK");

    let dt = chip
        .get_line(dt_pin)
        .expect("Failed to get DT line")
        .request(LineRequestFlags::INPUT, 0, "dt")
        .expect("Failed to request DT");

    let sw = chip
        .get_line(sw_pin)
        .expect("Failed to get SW line")
        .request(LineRequestFlags::INPUT, 0, "sw")
        .expect("Failed to request SW");

    let mut last_clk = clk.get_value().expect("Failed to read CLK");
    let mut last_sw = sw.get_value().expect("Failed to read SW");

    loop {
        let clk_value = clk.get_value().expect("Failed to read CLK");
        let dt_value = dt.get_value().expect("Failed to read DT");
        let sw_value = sw.get_value().expect("Failed to read SW");

        if clk_value != last_clk {
            if clk_value == 1 {
                if dt_value == 0 {
                    tx.send(RotaryEvent::Clockwise).unwrap();
                } else {
                    tx.send(RotaryEvent::CounterClockwise).unwrap();
                }
            }
            last_clk = clk_value;
        }

        if sw_value == 0 && last_sw == 1 {
            tx.send(RotaryEvent::ButtonPress).unwrap();
        }
        last_sw = sw_value;

        thread::sleep(Duration::from_millis(10));
    }
}
