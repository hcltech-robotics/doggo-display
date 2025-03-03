use linux_embedded_hal::I2cdev;
use std::error::Error;

pub fn get_i2c_bus() -> Result<I2cdev, Box<dyn Error>> {
    let dev = I2cdev::new("/dev/i2c-1")?;
    Ok(dev)
}
