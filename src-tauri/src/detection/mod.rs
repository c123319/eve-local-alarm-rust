pub mod hsv;
pub mod engine;
pub mod validation;

pub use engine::{DetectionEngine, DetectionResult, RuleMatchResult};
pub use hsv::{rgba_pixel_to_hsv, count_matching_pixels};
pub use validation::validate_color_match_config;
