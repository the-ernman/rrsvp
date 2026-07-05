use core::convert::Infallible;
use esp_idf_svc::sys::*;

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

use crate::board::*;

/// Pixels per DMA chunk (matching C++ txChunkBytes = 16 KB).
const TX_CHUNK_PIXELS: usize = (16 * 1024) / 2; // 8192 pixels

pub struct Display {
    spi: spi_device_handle_t,
    pub fb: Vec<u16>, // RGB565 framebuffer, LCD_W * LCD_H pixels
    tx_buf: Vec<u16>, // scratch buffer for one DMA chunk
}

impl Display {
    pub fn new() -> Self {
        unsafe { Self::init_hardware() }
    }

    /// Fill the entire framebuffer with a raw RGB565 u16 value and flush.
    pub fn fill_solid(&mut self, colour: u16) {
        for px in self.fb.iter_mut() {
            *px = colour;
        }
        self.flush();
    }



    /// Send a command and optional parameter bytes using the correct QSPI protocol.
    /// The AXS15231B requires opcode AND address to be sent on all 4 data lines.
    /// This matches the C++ sendCommand() which uses SPI_TRANS_MULTILINE_CMD | SPI_TRANS_MULTILINE_ADDR.
    unsafe fn send_command(spi: spi_device_handle_t, cmd: u8, data: &[u8]) {
        let mut trans: spi_transaction_t = core::mem::zeroed();
        trans.flags = (SPI_TRANS_MULTILINE_CMD | SPI_TRANS_MULTILINE_ADDR) as u32;
        trans.cmd = 0x02; // QSPI write opcode
        trans.addr = (cmd as u64) << 8;
        if !data.is_empty() {
            trans.length = data.len() * 8; // length in bits
            trans.__bindgen_anon_1.tx_buffer = data.as_ptr() as *const core::ffi::c_void;
        }
        let ret = spi_device_polling_transmit(spi, &mut trans);
        assert_eq!(ret, ESP_OK as i32, "send_command 0x{:02X} failed", cmd);
    }

    unsafe fn init_hardware() -> Self {
        log::info!("[display] init start");

        // --- Backlight off during init (active-low: HIGH = off) ---
        gpio_reset_pin(LCD_BL);
        gpio_set_direction(LCD_BL, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_level(LCD_BL, 1);

        // --- RST pin ---
        gpio_reset_pin(LCD_RST);
        gpio_set_direction(LCD_RST, gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_pull_mode(LCD_RST, gpio_pull_mode_t_GPIO_PULLUP_ONLY);

        // --- QSPI bus ---
        let bus_cfg = spi_bus_config_t {
            __bindgen_anon_1: spi_bus_config_t__bindgen_ty_1 {
                data0_io_num: LCD_D0,
            },
            __bindgen_anon_2: spi_bus_config_t__bindgen_ty_2 {
                data1_io_num: LCD_D1,
            },
            sclk_io_num: LCD_SCLK,
            __bindgen_anon_3: spi_bus_config_t__bindgen_ty_3 {
                data2_io_num: LCD_D2,
            },
            __bindgen_anon_4: spi_bus_config_t__bindgen_ty_4 {
                data3_io_num: LCD_D3,
            },
            data4_io_num: -1,
            data5_io_num: -1,
            data6_io_num: -1,
            data7_io_num: -1,
            // Match C++: max_transfer_sz = txChunkBytes + 8
            max_transfer_sz: (TX_CHUNK_PIXELS * 2 + 8) as i32,
            flags: SPICOMMON_BUSFLAG_MASTER | SPICOMMON_BUSFLAG_GPIO_PINS,
            intr_flags: 0,
            isr_cpu_id: esp_intr_cpu_affinity_t_ESP_INTR_CPU_AFFINITY_AUTO,
        };
        let ret = spi_bus_initialize(
            spi_host_device_t_SPI3_HOST,
            &bus_cfg,
            spi_common_dma_t_SPI_DMA_CH_AUTO,
        );
        log::info!("[display] spi_bus_initialize ret={}", ret);
        assert_eq!(ret, ESP_OK as i32, "spi_bus_initialize failed");

        // --- SPI device (matches C++ deviceConfig exactly) ---
        let mut dev_cfg: spi_device_interface_config_t = core::mem::zeroed();
        dev_cfg.command_bits = 8;
        dev_cfg.address_bits = 24;
        dev_cfg.mode = 3; // SPI_MODE3 (CPOL=1, CPHA=1)
        dev_cfg.clock_speed_hz = 40_000_000;
        dev_cfg.spics_io_num = LCD_CS;
        dev_cfg.flags = SPI_DEVICE_HALFDUPLEX as u32;
        dev_cfg.queue_size = 10;

        let mut spi: spi_device_handle_t = core::ptr::null_mut();
        let ret = spi_bus_add_device(spi_host_device_t_SPI3_HOST, &dev_cfg, &mut spi);
        log::info!("[display] spi_bus_add_device ret={}", ret);
        assert_eq!(ret, ESP_OK as i32, "spi_bus_add_device failed");

        // --- Hardware reset sequence ---
        gpio_set_level(LCD_RST, 1);
        std::thread::sleep(std::time::Duration::from_millis(30));
        gpio_set_level(LCD_RST, 0);
        std::thread::sleep(std::time::Duration::from_millis(250));
        gpio_set_level(LCD_RST, 1);
        std::thread::sleep(std::time::Duration::from_millis(30));
        log::info!("[display] hardware reset complete");

        // --- Panel init sequence (kQspiInit from axs15231b.cpp, with hardware landscape MADCTL) ---
        Self::send_command(spi, 0x11, &[]); // Sleep Out
        std::thread::sleep(std::time::Duration::from_millis(100));
        Self::send_command(spi, 0x36, &[0x00]); // MADCTL = 0x00 (portrait, no hardware rotation — matches C++)
        Self::send_command(spi, 0x3A, &[0x55]); // COLMOD: RGB565
        Self::send_command(spi, 0x11, &[]); // Sleep Out (second — required by AXS15231B)
        std::thread::sleep(std::time::Duration::from_millis(100));
        Self::send_command(spi, 0x29, &[]); // Display On
        std::thread::sleep(std::time::Duration::from_millis(100));
        log::info!("[display] panel init sequence complete");

        // --- Enable TCA9554 backlight power via system I2C (SDA=47, SCL=48) ---
        // The Waveshare board uses a TCA9554 GPIO expander to gate the backlight boost converter.
        // Pin 1 = backlight enable, Pin 6 = system enable. Both must be HIGH (output) to power the backlight.
        {
            let mut sys_i2c_cfg: i2c_config_t = core::mem::zeroed();
            sys_i2c_cfg.mode = i2c_mode_t_I2C_MODE_MASTER;
            sys_i2c_cfg.sda_io_num = SYS_I2C_SDA;
            sys_i2c_cfg.scl_io_num = SYS_I2C_SCL;
            sys_i2c_cfg.sda_pullup_en = false;
            sys_i2c_cfg.scl_pullup_en = false;
            sys_i2c_cfg.__bindgen_anon_1.master.clk_speed = SYS_I2C_CLOCK_HZ;

            let r1 = i2c_param_config(TCA9554_I2C_PORT as i2c_port_t, &sys_i2c_cfg);
            log::info!("[display] TCA9554 i2c_param_config ret={}", r1);
            if r1 == ESP_OK as i32 {
                let r2 = i2c_driver_install(
                    TCA9554_I2C_PORT as i2c_port_t,
                    i2c_mode_t_I2C_MODE_MASTER,
                    0,
                    0,
                    0,
                );
                log::info!("[display] TCA9554 i2c_driver_install ret={}", r2);
                if r2 == ESP_OK as i32 {
                    // Set output register first (pins 1 & 6 HIGH before configuring as outputs)
                    let out_data: [u8; 2] = [0x01, 0xFF];
                    let r3 = i2c_master_write_to_device(
                        TCA9554_I2C_PORT as i2c_port_t,
                        TCA9554_ADDR,
                        out_data.as_ptr(),
                        out_data.len(),
                        100,
                    );
                    log::info!("[display] TCA9554 output reg write ret={}", r3);
                    // Configure pins 1 and 6 as outputs (0xBD = ~(1<<1) & ~(1<<6))
                    let cfg_data: [u8; 2] = [0x03, 0xBD];
                    let r4 = i2c_master_write_to_device(
                        TCA9554_I2C_PORT as i2c_port_t,
                        TCA9554_ADDR,
                        cfg_data.as_ptr(),
                        cfg_data.len(),
                        100,
                    );
                    log::info!("[display] TCA9554 config reg write ret={}", r4);
                }
            }
        }

        // --- Backlight on (active-low: LOW = full brightness) ---
        gpio_set_level(LCD_BL, 0);
        log::info!("[display] backlight ON (GPIO{}=LOW)", LCD_BL);

        let fb = vec![0u16; LCD_W * LCD_H];
        let tx_buf = Vec::with_capacity(TX_CHUNK_PIXELS);
        log::info!(
            "[display] init complete, fb={}x{} ({} pixels)",
            LCD_W,
            LCD_H,
            LCD_W * LCD_H
        );
        Display { spi, fb, tx_buf }
    }

    /// Flush the framebuffer to the display using 16 KB DMA chunks.
    /// The AXS15231B requires RAMWR (0x2C) for the first block and 0x3C
    /// for every later block; a single monolithic write only covers the
    /// first few rows. We also transpose the 640×172 framebuffer to the
    /// native 172×640 panel order on the fly.
    pub fn flush(&mut self) {
        const NATIVE_W: usize = PANEL_NATIVE_W;
        const NATIVE_H: usize = PANEL_NATIVE_H;
        const ROWS_PER_DMA: usize = TX_CHUNK_PIXELS / NATIVE_W; // ≈ 47 rows

        unsafe {
            // CASET: physical portrait column range 0 to 171.
            let ec: u16 = (NATIVE_W - 1) as u16;
            let caset: [u8; 4] = [0x00, 0x00, (ec >> 8) as u8, ec as u8];
            Self::send_command(self.spi, 0x2A, &caset);
            // Note: no RASET — C++ never sends RASET, the full row range is the default

            for row_block_start in (0..NATIVE_H).step_by(ROWS_PER_DMA) {
                let rows_this_block = ROWS_PER_DMA.min(NATIVE_H - row_block_start);
                let pixels_this_block = rows_this_block * NATIVE_W;

                // --- transpose needed pixels into tx_buf ---
                self.tx_buf.clear();
                for native_y in row_block_start..(row_block_start + rows_this_block) {
                    for native_x in 0..NATIVE_W {
                        let fb_x = native_y;
                        let fb_y = (LCD_H - 1) - native_x;
                        let idx = fb_y * LCD_W + fb_x;
                        self.tx_buf.push(self.fb[idx]);
                    }
                }

                // --- send this block over QSPI ---
                let is_first_block = row_block_start == 0;
                let mut offset = 0usize;
                let mut first_chunk = true;

                while offset < pixels_this_block {
                    let chunk = (pixels_this_block - offset).min(TX_CHUNK_PIXELS);
                    let pixels_ptr = self.tx_buf[offset..].as_ptr() as *const core::ffi::c_void;
                    let length_bits = (chunk * 16) as usize;

                    if first_chunk {
                        let addr = if is_first_block {
                            0x002C00u64 // RAMWR
                        } else {
                            0x003C00u64 // continuation
                        };
                        let mut trans: spi_transaction_t = core::mem::zeroed();
                        trans.flags = SPI_TRANS_MODE_QIO as u32;
                        trans.cmd = 0x32; // QIO write opcode
                        trans.addr = addr;
                        trans.length = length_bits;
                        trans.__bindgen_anon_1.tx_buffer = pixels_ptr;
                        let ret = spi_device_polling_transmit(self.spi, &mut trans);
                        assert_eq!(ret, ESP_OK as i32, "pixel flush first chunk failed");
                        first_chunk = false;
                    } else {
                        let mut trans: spi_transaction_ext_t = core::mem::zeroed();
                        trans.base.flags = (SPI_TRANS_MODE_QIO
                            | SPI_TRANS_VARIABLE_CMD
                            | SPI_TRANS_VARIABLE_ADDR
                            | SPI_TRANS_VARIABLE_DUMMY)
                            as u32;
                        trans.command_bits = 0;
                        trans.address_bits = 0;
                        trans.dummy_bits = 0;
                        trans.base.length = length_bits;
                        trans.base.__bindgen_anon_1.tx_buffer = pixels_ptr;
                        let ret = spi_device_polling_transmit(
                            self.spi,
                            core::ptr::addr_of_mut!(trans.base),
                        );
                        assert_eq!(ret, ESP_OK as i32, "pixel flush chunk failed");
                    }

                    offset += chunk;
                }
            }
        }
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        Size::new(LCD_W as u32, LCD_H as u32)
    }
}

impl DrawTarget for Display {
    type Color = Rgb565;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(Point { x, y }, color) in pixels {
            if x >= 0 && x < LCD_W as i32 && y >= 0 && y < LCD_H as i32 {
                let idx = y as usize * LCD_W + x as usize;
                self.fb[idx] = color.into_storage().to_be();
            }
        }
        Ok(())
    }
}
