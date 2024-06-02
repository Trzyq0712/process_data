use std::path::Path;

use serde::Deserialize;

use crate::point::Point;

const fn led_to_point(led: usize) -> Point {
    let x = led % 6;
    let y = led / 6;
    Point {
        x: x as u32 * 500 + 250,
        y: y as u32 * 500 + 250,
    }
}

pub struct CleanAugmentConfig {
    pub clean_dist: u32,
    pub augm_dist: u32,
    pub continuity_thresh: f32,
    pub led_count: usize,
    pub height: u32,
    pub led_positions: Vec<Point>,
    pub half_power_semiangle: f32,
    pub lambertian_order: f32,
    pub prop_loss_func: fn(f32) -> f32,
    pub augm_min_neighbors: usize,
    pub darkness_penalty: f32,
    pub led_fov: f32,
    pub augm_min_neighbors2: usize,
}

impl Default for CleanAugmentConfig {
    fn default() -> Self {
        let hpsa = 15.0_f32.to_radians();
        let m = -f32::ln(2.0) / hpsa.cos().ln();
        CleanAugmentConfig {
            clean_dist: 30,
            augm_dist: 50,
            continuity_thresh: 0.08,
            led_count: 36,
            height: 176 * 10,
            led_positions: (0..36).map(led_to_point).collect(),
            half_power_semiangle: hpsa,
            lambertian_order: m,
            prop_loss_func: |d| d.powi(-2),
            augm_min_neighbors: 10,
            darkness_penalty: 3.0,
            led_fov: 30.0_f32.to_radians(),
            augm_min_neighbors2: 4,
        }
    }
}

impl CleanAugmentConfig {
    pub fn from_file(path: &Path) -> Result<CleanAugmentConfig, Box<dyn std::error::Error>> {
        let config_file = std::fs::read_to_string(path)?;
        let config_builder: ConfigBuilder = toml::from_str(&config_file)?;
        Ok(config_builder.build())
    }
}

#[derive(Deserialize, Debug)]
struct ConfigBuilder {
    clean_dist: Option<u32>,
    augm_dist: Option<u32>,
    continuity_thresh: Option<f32>,
    led_count: Option<usize>,
    height: Option<u32>,
    led_positions: Option<Vec<[f32; 2]>>,
    half_power_semiangle: Option<f32>,
    augm_min_neighbors: Option<usize>,
    darkness_penalty: Option<f32>,
}

impl ConfigBuilder {
    fn build(self) -> CleanAugmentConfig {
        let default = CleanAugmentConfig::default();
        CleanAugmentConfig {
            clean_dist: self.clean_dist.unwrap_or(default.clean_dist),
            augm_dist: self.augm_dist.unwrap_or(default.augm_dist),
            continuity_thresh: self.continuity_thresh.unwrap_or(default.continuity_thresh),
            led_count: self.led_count.unwrap_or(default.led_count),
            height: self.height.unwrap_or(default.height),
            led_positions: self
                .led_positions
                .map(|v| {
                    v.into_iter()
                        .map(|[x, y]| Point {
                            x: x as u32,
                            y: y as u32,
                        })
                        .collect()
                })
                .unwrap_or(default.led_positions),
            half_power_semiangle: self
                .half_power_semiangle
                .unwrap_or(default.half_power_semiangle),
            lambertian_order: default.lambertian_order,
            prop_loss_func: default.prop_loss_func,
            augm_min_neighbors: self
                .augm_min_neighbors
                .unwrap_or(default.augm_min_neighbors),
            darkness_penalty: self.darkness_penalty.unwrap_or(default.darkness_penalty),
            led_fov: default.led_fov,
            augm_min_neighbors2: default.augm_min_neighbors2,
        }
    }
}

pub type Config = CleanAugmentConfig;

pub fn pb_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-")
}

pub fn pb_style2() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::default_bar()
        .template("{spinner:.green} [{pos}/{len}] {msg}")
        .unwrap()
        .progress_chars("#>-")
}
