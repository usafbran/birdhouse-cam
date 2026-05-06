/// Bird detection module.
///
/// Uses a two-tier approach:
/// 1. On-device motion detection via frame differencing to detect activity
/// 2. Optional offload to a classification server for species identification
///
/// Motion detection compares consecutive JPEG frame sizes and optionally
/// pixel data to determine if something has changed in the scene. This is
/// lightweight enough to run on the ESP32 at capture rate.
use anyhow::Result;
use log::{debug, info, warn};

use crate::config;

#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub motion_detected: bool,
    pub motion_score: f32,
    pub species: Option<String>,
    pub confidence: Option<f32>,
}

pub struct MotionDetector {
    prev_frame_size: Option<usize>,
    prev_frame_hash: Option<u64>,
    threshold: f32,
}

impl MotionDetector {
    pub fn new() -> Self {
        Self {
            prev_frame_size: None,
            prev_frame_hash: None,
            threshold: config::MOTION_PIXEL_PERCENT,
        }
    }

    /// Detect motion by comparing the current frame against the previous one.
    ///
    /// Uses two heuristics:
    /// 1. JPEG file size change (significant size changes indicate scene changes)
    /// 2. Simple hash of sampled pixels from the frame data
    pub fn detect(&mut self, frame_data: &[u8]) -> DetectionResult {
        let current_size = frame_data.len();
        let current_hash = self.compute_frame_hash(frame_data);

        let (motion_detected, motion_score) = match (self.prev_frame_size, self.prev_frame_hash) {
            (Some(prev_size), Some(prev_hash)) => {
                let size_ratio = if prev_size > 0 {
                    (current_size as f64 - prev_size as f64).abs() / prev_size as f64 * 100.0
                } else {
                    0.0
                };

                let hash_diff = (current_hash ^ prev_hash).count_ones() as f32;
                let hash_score = hash_diff / 64.0 * 100.0;

                // Combine both signals
                let combined_score = (size_ratio as f32 * 0.4) + (hash_score * 0.6);
                let detected = combined_score > self.threshold;

                debug!(
                    "Motion detection - size_ratio: {:.1}%, hash_score: {:.1}%, combined: {:.1}%, threshold: {:.1}%",
                    size_ratio, hash_score, combined_score, self.threshold
                );

                (detected, combined_score)
            }
            _ => {
                debug!("First frame captured, no motion detection yet");
                (false, 0.0)
            }
        };

        self.prev_frame_size = Some(current_size);
        self.prev_frame_hash = Some(current_hash);

        if motion_detected {
            info!("Motion detected! Score: {:.1}%", motion_score);
        }

        DetectionResult {
            motion_detected,
            motion_score,
            species: None,
            confidence: None,
        }
    }

    /// Compute a simple hash of the frame data by sampling bytes at regular
    /// intervals. This is fast and gives a reasonable fingerprint of the image
    /// content for change detection purposes.
    fn compute_frame_hash(&self, data: &[u8]) -> u64 {
        if data.is_empty() {
            return 0;
        }

        let mut hash: u64 = 0;
        let step = (data.len() / 64).max(1);

        for i in 0..64 {
            let idx = (i * step).min(data.len() - 1);
            let bit = if data[idx] > 128 { 1u64 } else { 0u64 };
            hash |= bit << i;
        }

        hash
    }
}

/// Remote bird classifier that sends images to an external server
/// for species identification.
pub struct RemoteClassifier {
    server_url: String,
}

impl RemoteClassifier {
    pub fn new() -> Option<Self> {
        let url = config::CLASSIFICATION_SERVER_URL;
        if url.is_empty() {
            info!("No classification server configured, species identification disabled");
            return None;
        }

        info!("Classification server configured: {}", url);
        Some(Self {
            server_url: url.to_string(),
        })
    }

    /// Send a JPEG frame to the classification server and get the species.
    ///
    /// Expected server API:
    ///   POST /classify
    ///   Content-Type: image/jpeg
    ///   Body: raw JPEG data
    ///
    ///   Response: { "species": "Blue Jay", "confidence": 0.92 }
    pub fn classify(&self, jpeg_data: &[u8]) -> Result<(String, f32)> {
        use esp_idf_svc::http::client::{Configuration, EspHttpConnection};

        let config = Configuration {
            buffer_size: Some(2048),
            buffer_size_tx: Some(jpeg_data.len() + 256),
            timeout: Some(std::time::Duration::from_secs(10)),
            ..Default::default()
        };

        let mut connection = EspHttpConnection::new(&config)?;

        let url = format!("{}/classify", self.server_url);
        let content_len = jpeg_data.len().to_string();
        let headers = [
            ("Content-Type", "image/jpeg"),
            ("Content-Length", content_len.as_str()),
        ];

        connection.initiate_request(embedded_svc::http::Method::Post, &url, &headers)?;

        use std::io::Write;
        connection.write_all(jpeg_data)?;

        connection.initiate_response()?;
        let status = connection.status();
        if status != 200 {
            warn!("Classification server returned status {}", status);
            anyhow::bail!("Classification failed with status {}", status);
        }

        use std::io::Read;
        let mut buf = [0u8; 512];
        let bytes_read = connection.read(&mut buf)?;
        let body = core::str::from_utf8(&buf[..bytes_read])?;

        // Parse JSON response
        let parsed: serde_json::Value = serde_json::from_str(body)?;
        let species = parsed["species"].as_str().unwrap_or("unknown").to_string();
        let confidence = parsed["confidence"].as_f64().unwrap_or(0.0) as f32;

        info!(
            "Classification result: {} ({:.0}% confidence)",
            species,
            confidence * 100.0
        );

        Ok((species, confidence))
    }
}
