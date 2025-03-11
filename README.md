## 🐶 Doggo Display Daemon  

**Doggo Display** is a simple daemon for displaying system information on an I2C OLED screen. It supports a rotary encoder for navigation and button-press actions.  

### **✨ Features**
✅ **Configurable Display Blocks** – Show static text, dynamic content (IP, CPU usage), or execute commands.  
✅ **Rotary Encoder Support** – Navigate between display blocks and trigger actions.  
✅ **Systemd Integration** – Runs on boot as a background service.  
✅ **Simple Installation** – Just download the latest release and run `setup.sh`.  

---

## 📦 **Installation** (Recommended)  

### **1️⃣ Download the Latest Release**  
```bash
wget https://github.com/hcltech-robotics/doggo-display/releases/latest/download/doggo-display.tar.gz
tar -xvf doggo-display.tar.gz
cd doggo-display
```

### **2️⃣ Run the Setup Script**  
```bash
chmod +x setup.sh
sudo ./setup.sh
```

### **3️⃣ Verify the Service is Running**  
```bash
systemctl status doggo-display
```

---

## ⚙️ **Configuration**  

The daemon reads its settings from `/etc/doggo-display/config.toml`.  

### **Example `config.toml`**
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

### **📌 Parameters**
- `i2c_bus` → Path to the I2C bus.  
- `chip` → Path to GPIO chip for rotary encoder input.  
- `clk, dt, sw` → GPIO pin numbers for the rotary encoder.  
- `blocks` → Defines the content displayed on the screen.  
  - `top_line, second_line` → Text to display (or output of a shell command).  
  - `button_enabled` → If `true`, pressing the encoder button runs `function_to_run`.  
  - `refresh_interval` → (Optional) Auto-refresh time in seconds.  

> **Editing the Config:**  
> ```bash
> sudo nano /etc/doggo-display/config.toml
> sudo systemctl restart doggo-display  # Apply changes
> ```

---

## 🔄 **How to Update**  
To update to the latest version:  
```bash
wget https://github.com/hcltech-robotics/doggo-display/releases/latest/download/doggo-display.tar.gz
tar -xvf doggo-display.tar.gz
cd doggo-display
sudo ./setup.sh
```

---

## 📜 **Logging**  
- Logs are stored in `/var/log/doggo-display/doggo-display.log`.  
- Check logs with:  
  ```bash
  journalctl -u doggo-display --follow
  ```

---

## 📜 **License**  
This project is licensed under the **MIT License**.  

