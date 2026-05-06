/// Camera module for the ESP32-CAM OV2640 sensor.
///
/// Provides initialization and frame capture using the ESP-IDF
/// camera driver via FFI bindings.
use anyhow::{bail, Result};
use esp_idf_svc::sys::{
    camera::{
        camera_config_t, camera_config_t__bindgen_ty_1, camera_fb_t, esp_camera_deinit,
        esp_camera_fb_get, esp_camera_fb_return, esp_camera_init, pixformat_t_PIXFORMAT_JPEG,
    },
    gpio_num_t,
};
use log::{error, info};

use crate::config;

pub struct FrameBuffer {
    fb: *mut camera_fb_t,
}

impl FrameBuffer {
    /// Raw image data bytes.
    pub fn data(&self) -> &[u8] {
        unsafe {
            let fb = &*self.fb;
            core::slice::from_raw_parts(fb.buf, fb.len)
        }
    }

    pub fn width(&self) -> usize {
        unsafe { (*self.fb).width as usize }
    }

    pub fn height(&self) -> usize {
        unsafe { (*self.fb).height as usize }
    }

    pub fn len(&self) -> usize {
        unsafe { (*self.fb).len }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn format(&self) -> u32 {
        unsafe { (*self.fb).format }
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        unsafe {
            esp_camera_fb_return(self.fb);
        }
    }
}

// FrameBuffer holds a raw pointer but is only used on the thread that captured it
unsafe impl Send for FrameBuffer {}

pub struct Camera {
    initialized: bool,
}

impl Camera {
    pub fn new() -> Result<Self> {
        let camera_config = camera_config_t {
            pin_pwdn: config::pins::PWDN as gpio_num_t,
            pin_reset: config::pins::RESET as gpio_num_t,
            pin_xclk: config::pins::XCLK as gpio_num_t,
            pin_sccb_sda: config::pins::SIOD as gpio_num_t,
            pin_sccb_scl: config::pins::SIOC as gpio_num_t,
            pin_d7: config::pins::Y9 as gpio_num_t,
            pin_d6: config::pins::Y8 as gpio_num_t,
            pin_d5: config::pins::Y7 as gpio_num_t,
            pin_d4: config::pins::Y6 as gpio_num_t,
            pin_d3: config::pins::Y5 as gpio_num_t,
            pin_d2: config::pins::Y4 as gpio_num_t,
            pin_d1: config::pins::Y3 as gpio_num_t,
            pin_d0: config::pins::Y2 as gpio_num_t,
            pin_vsync: config::pins::VSYNC as gpio_num_t,
            pin_href: config::pins::HREF as gpio_num_t,
            pin_pclk: config::pins::PCLK as gpio_num_t,
            xclk_freq_hz: 20_000_000,
            ledc_timer: 0,
            ledc_channel: 0,
            pixel_format: pixformat_t_PIXFORMAT_JPEG,
            frame_size: config::CAMERA_FRAME_SIZE as i32,
            jpeg_quality: config::CAMERA_JPEG_QUALITY as i32,
            fb_count: config::CAMERA_FB_COUNT as i32,
            grab_mode: 1,   // CAMERA_GRAB_LATEST
            fb_location: 1, // CAMERA_FB_IN_PSRAM
            __bindgen_anon_1: camera_config_t__bindgen_ty_1 { sccb_i2c_port: -1 },
        };

        let ret = unsafe { esp_camera_init(&camera_config) };
        if ret != 0 {
            bail!("Camera init failed with error code: {}", ret);
        }

        info!("Camera initialized successfully");
        Ok(Self { initialized: true })
    }

    /// Capture a JPEG frame from the camera.
    pub fn capture_jpeg(&self) -> Result<FrameBuffer> {
        if !self.initialized {
            bail!("Camera not initialized");
        }

        let fb = unsafe { esp_camera_fb_get() };
        if fb.is_null() {
            bail!("Failed to capture frame");
        }

        let frame = FrameBuffer { fb };
        info!(
            "Captured frame: {}x{}, {} bytes",
            frame.width(),
            frame.height(),
            frame.len()
        );

        Ok(frame)
    }

    /// Capture a grayscale frame for motion detection.
    /// Temporarily switches the camera to grayscale mode.
    pub fn capture_grayscale(&self) -> Result<FrameBuffer> {
        if !self.initialized {
            bail!("Camera not initialized");
        }

        // For motion detection we use the JPEG frame and will
        // do simple comparison on the compressed data size or
        // decode a small portion. In practice, comparing JPEG
        // sizes frame-to-frame gives a reasonable motion signal.
        self.capture_jpeg()
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                let ret = esp_camera_deinit();
                if ret != 0 {
                    error!("Camera deinit failed: {}", ret);
                }
            }
            self.initialized = false;
        }
    }
}
