// SPDX-License-Identifier: MIT
// Copyright (c) 2025 HCLTech Ltd. All rights reserved.
// See the LICENSE file in the project root for more details.

use embedded_hal::i2c::I2c;
use linux_embedded_hal::I2cdev;
use std::thread::sleep;
use std::time::Duration;
use tracing::{debug, error, info};

const LCD_ADDR: u8 = 0x27;
const LCD_BACKLIGHT: u8 = 0x08; // Backlight ON
const LCD_ENABLE: u8 = 0x04; // Enable bit
const LCD_CMD: u8 = 0x00; // RS = 0 (command mode)
const LCD_DATA: u8 = 0x01; // RS = 1 (data mode)

pub struct Display {
    i2c: I2cdev,
}

impl Display {
    pub fn new(i2c: I2cdev) -> Self {
        let mut dplay = Display { i2c };
        if let Err(e) = dplay.init() {
            error!("Failed to initialize display: {}", e);
        } else {
            info!("Display initialized successfully");
        }
        dplay
    }

    fn send_nibble(&mut self, nibble: u8, mode: u8) -> Result<(), Box<dyn std::error::Error>> {
        let data = nibble | mode | LCD_BACKLIGHT;

        self.i2c.write(LCD_ADDR, &[data | LCD_ENABLE])?; // E=1
        sleep(Duration::from_micros(500)); // Small delay
        self.i2c.write(LCD_ADDR, &[data & !LCD_ENABLE])?; // E=0
        sleep(Duration::from_micros(100));
        Ok(())
    }

    fn send_byte(&mut self, byte: u8, mode: u8) -> Result<(), Box<dyn std::error::Error>> {
        self.send_nibble(byte & 0xF0, mode)?; // Send high nibble
        self.send_nibble((byte << 4) & 0xF0, mode)?; // Send low nibble
        Ok(())
    }

    fn send_command(&mut self, cmd: u8) -> Result<(), Box<dyn std::error::Error>> {
        self.send_byte(cmd, LCD_CMD)?;
        sleep(Duration::from_millis(2)); // Some commands need a delay
        Ok(())
    }

    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing display...");

        sleep(Duration::from_millis(50)); // Power-on delay
        self.send_nibble(0x30, LCD_CMD)?; // Wake up
        sleep(Duration::from_millis(5));
        self.send_nibble(0x30, LCD_CMD)?;
        sleep(Duration::from_millis(5));
        self.send_nibble(0x30, LCD_CMD)?;
        sleep(Duration::from_millis(5));
        self.send_nibble(0x20, LCD_CMD)?; // Switch to 4-bit mode

        self.send_command(0x28)?; // Function set: 4-bit, 2 lines, 5x8 font
        self.send_command(0x0C)?; // Display on, cursor off
        self.send_command(0x06)?; // Entry mode: Increment cursor
        self.send_command(0x01)?; // Clear display
        info!("Display initialized");
        Ok(())
    }

    pub fn write_text(
        &mut self,
        line1: &str,
        line2: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Writing text to display");

        self.send_command(0x01)?; // Clear display
        sleep(Duration::from_millis(2));

        // Write first line
        self.send_command(0x80)?; // Move to first line (0x80 is DDRAM address for line 1)
        debug!("Writing to first line: {}", line1);
        for c in line1.bytes() {
            self.send_byte(c, LCD_DATA)?;
        }

        // Move to the second line
        self.send_command(0xC0)?; // Move to second line (0xC0 is DDRAM address for line 2)
        debug!("Writing to second line: {}", line2);
        for c in line2.bytes() {
            self.send_byte(c, LCD_DATA)?;
        }

        info!("Text written to display successfully");
        Ok(())
    }
}
