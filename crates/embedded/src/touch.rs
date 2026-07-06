use esp_idf_svc::sys::*;

use crate::board::*;

/// A decoded touch sample in landscape coordinates (0..640 x 0..172).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TPoint {
    pub x: u16,
    pub y: u16,
}

/// Write-read command to trigger the AXS15231B to report touch data.
const READ_CMD: [u8; 11] = [
    0xB5, 0xAB, 0xA5, 0x5A, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
];
const PACKET_LEN: usize = 8;
const I2C_PORT: i2c_port_t = 0; // I2C_NUM_0

/// AXS15231B touch driver.
pub struct TouchDriver;

impl TouchDriver {
    /// Initialise the I2C bus for the touch controller.
    /// SAFETY: called once at startup before any other I2C use on port 0.
    pub fn new() -> Self {
        unsafe { Self::init_hardware() };
        TouchDriver
    }

    unsafe fn init_hardware() {
        // Use the legacy ESP-IDF I2C driver API (i2c_param_config + i2c_driver_install).
        // The legacy i2c_config_t does not require the i2c_clock_source_t enum, which is
        // only a C preprocessor macro and is not present in the bindgen output.
        let mut config: i2c_config_t = core::mem::zeroed();
        config.mode = i2c_mode_t_I2C_MODE_MASTER;
        config.sda_io_num = TOUCH_SDA;
        config.scl_io_num = TOUCH_SCL;
        config.sda_pullup_en = false;
        config.scl_pullup_en = false;
        // Set master clock speed through the anonymous union.
        config.__bindgen_anon_1.master.clk_speed = TOUCH_I2C_CLOCK_HZ;

        let ret = i2c_param_config(I2C_PORT, &config);
        assert_eq!(ret, ESP_OK as i32, "i2c_param_config failed");

        let ret = i2c_driver_install(I2C_PORT, i2c_mode_t_I2C_MODE_MASTER, 0, 0, 0);
        assert_eq!(ret, ESP_OK as i32, "i2c_driver_install failed");
    }

    /// Read the current touch state. Returns Some(TPoint) if touched, None otherwise.
    pub fn read(&mut self) -> Option<TPoint> {
        let mut buf = [0u8; PACKET_LEN];
        // i2c_master_write_read_device performs a write then a repeated-start read
        // without releasing the bus — equivalent to the C++ Wire.endTransmission(false)
        // followed by Wire.requestFrom().
        let ret = unsafe {
            i2c_master_write_read_device(
                I2C_PORT,
                TOUCH_I2C_ADDR,
                READ_CMD.as_ptr(),
                READ_CMD.len(),
                buf.as_mut_ptr(),
                PACKET_LEN,
                100, // timeout in FreeRTOS ticks (1 tick = 1 ms with CONFIG_FREERTOS_HZ=1000)
            )
        };
        if ret != ESP_OK as i32 {
            return None;
        }
        self.decode(&buf)
    }

    fn decode(&self, data: &[u8; PACKET_LEN]) -> Option<TPoint> {
        let points = data[1];
        if points == 0 || points > 4 {
            return None;
        }

        // AXS15231B: long-axis (raw_long) = portrait Y (0..640), short-axis (raw_short) = portrait X (0..172).
        let raw_long = (((data[2] & 0x0F) as u16) << 8) | data[3] as u16;
        let raw_short = (((data[4] & 0x0F) as u16) << 8) | data[5] as u16;

        // Map to landscape screen coordinates: (0,0) = top-left, x increases right, y increases down.
        // Physical panel has kPanelMemoryRotated180=true, so raw (0,0) is at physical bottom-right.
        // Flip both axes to correct orientation.
        let lx = (TOUCH_PANEL_H - 1).saturating_sub(raw_long.min(TOUCH_PANEL_H - 1));
        let ly = (TOUCH_PANEL_W - 1).saturating_sub(raw_short.min(TOUCH_PANEL_W - 1));

        Some(TPoint { x: lx, y: ly })
    }
}
