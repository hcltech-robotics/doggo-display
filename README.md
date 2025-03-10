# Doggo Display Daemon

This project provides a simple daemon that displays information on an I2C OLED screen and interacts with a rotary encoder. It reads configurations from a `config.toml` file, handles rotary encoder input, executes commands on button presses, and updates the display.

## Features

- **Configurable display blocks**: Each block can display static text, dynamic content (e.g., IP address, CPU usage), or execute shell commands.
- **Rotary encoder**: Navigate through display blocks and trigger actions by turning the encoder or pressing the button.
- **Systemd integration**: The application can be managed as a service, starting on boot and running in the background.

## Configuration

The configuration file (`config.toml`) allows you to define the hardware setup (GPIO pins, I2C bus) and the blocks to display on the OLED screen.

### Example `config.toml`

```toml
[hardware]
i2c_bus = "/dev/i2c-1"
chip = "/dev/gpiochip4"
clk = 17
dt = 27
sw = 22

[[blocks]]
top_line = "IP Address:"
second_line = "!hostname -I"
button_enabled = true
function_to_run = "echo 'Button Pressed'"
refresh_interval = 0

[[blocks]]
top_line = "CPU Usage:"
second_line = "!top -bn1 | grep 'Cpu(s)' | awk '{print $2}'"
button_enabled = false
function_to_run = ""
refresh_interval = 2

[[blocks]]
top_line = "Shutdown"
second_line = ""
button_enabled = true
function_to_run = "shutdown"
refresh_interval = 0
```

### Parameters

- **i2c_bus**: Path to the I2C bus for communication with the OLED display.
- **chip**: Path to the GPIO chip for managing rotary encoder pins.
- **clk, dt, sw**: GPIO pin assignments for the rotary encoder (clock, data, and switch pins).
- **blocks**: Each block represents a display unit, which can either show text or execute a command when the button is pressed.

## Installation

1. **Clone the repository**:
    ```bash
    git clone https://github.com/yourusername/doggo-display.git
    cd doggo-display
    ```

2. **Build the project**:
    ```bash
    cargo build --release
    ```

3. **Install dependencies**:
    - Ensure you have `tracing`, `gpio-cdev`, and any other dependencies installed.
    - Make sure your system has access to the I2C and GPIO interfaces.

4. **Set up systemd**:
    - Create a systemd service file for your application:
    ```bash
    sudo nano /etc/systemd/system/doggo-display.service
    ```
    Example service file:
    ```ini
    [Unit]
    Description=Doggo Display Daemon
    After=network.target

    [Service]
    ExecStart=/path/to/your/executable
    WorkingDirectory=/path/to/your/project/directory
    Restart=always
    User=youruser
    Group=yourgroup

    [Install]
    WantedBy=multi-user.target
    ```
    - Reload the systemd daemon and enable the service:
    ```bash
    sudo systemctl daemon-reload
    sudo systemctl enable doggo-display.service
    sudo systemctl start doggo-display.service
    ```

## Usage

Once the daemon is running, the OLED display will show the configured blocks. You can navigate through the blocks using the rotary encoder and interact with them by pressing the encoder button.

### Button Press Actions

- If the block has a shell command assigned, it will execute on button press.
- The shutdown block can be used to shut down the system.

## Logging

- Logs are stored in `/var/log/doggo-display/doggo-display.log`.
- Logs are also available via `journalctl` when running as a systemd service.

## License

This project is licensed under the MIT License.