// ── Display (AXS15231B QSPI) ─────────────────────────────────────────────────
pub const LCD_CS: i32 = 9;
pub const LCD_SCLK: i32 = 10;
pub const LCD_D0: i32 = 11;
pub const LCD_D1: i32 = 12;
pub const LCD_D2: i32 = 13;
pub const LCD_D3: i32 = 14;
pub const LCD_RST: i32 = 21;
pub const LCD_BL: i32 = 8;

// Panel native portrait: 172 wide, 640 tall.
// Software framebuffer is landscape: 640 × 172.
// Rotation to portrait is done in flush() via transpose.
pub const LCD_W: usize = 640;
pub const LCD_H: usize = 172;

pub const PANEL_NATIVE_W: usize = 172;
pub const PANEL_NATIVE_H: usize = 640;

// ── Touch (AXS15231B I2C) ─────────────────────────────────────────────────────
pub const TOUCH_I2C_ADDR: u8 = 0x3B;
pub const TOUCH_SDA: i32 = 17;
pub const TOUCH_SCL: i32 = 18;
pub const TOUCH_I2C_CLOCK_HZ: u32 = 300_000;

// Panel native dimensions (portrait, used for touch coordinate decode).
pub const TOUCH_PANEL_W: u16 = 172;
pub const TOUCH_PANEL_H: u16 = 640;

// ── Buttons ───────────────────────────────────────────────────────────────────
pub const BTN_BOOT: i32 = 0;
pub const BTN_POWER: i32 = 16;

// ── System I2C bus (TCA9554 GPIO expander, IMU) ───────────────────────────────
pub const SYS_I2C_SDA: i32 = 47;
pub const SYS_I2C_SCL: i32 = 48;
pub const SYS_I2C_CLOCK_HZ: u32 = 300_000;

// ── TCA9554 GPIO expander ─────────────────────────────────────────────────────
pub const TCA9554_I2C_PORT: i32 = 1; // I2C_NUM_1
pub const TCA9554_ADDR: u8 = 0x20;
pub const TCA9554_BL_ENABLE_PIN: u8 = 1; // bit 1 of TCA9554 output
pub const TCA9554_SYS_ENABLE_PIN: u8 = 6; // bit 6 of TCA9554 output
