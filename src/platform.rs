use linux_embedded_hal::I2cdev;
use std::error::Error;
use tracing::{debug, error, info}; // Import tracing macros

pub fn get_i2c_bus(bus: &str) -> Result<I2cdev, Box<dyn Error>> {
    debug!("Attempting to open I2C bus: {}", bus);

    match I2cdev::new(bus) {
        Ok(dev) => {
            info!("Successfully initialized I2C bus: {}", bus);
            Ok(dev)
        }
        Err(e) => {
            error!("Failed to open I2C bus {}: {}", bus, e);
            Err(Box::new(e))
        }
    }
}
