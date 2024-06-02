use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use crate::augment::augment_point;
use crate::config::{pb_style, Config};
use crate::point::Point;
use crate::point_map::PointMap;
use crate::rss_record::{RssArr, RssRecord};

pub fn clean_records_stg1(raw_records: Vec<RssRecord>, config: &Config) -> Vec<RssRecord> {
    let point_map = PointMap::from_raw_records(raw_records);
    let points = point_map.all_points();
    let stg1 = indicatif::ProgressBar::new(points.len() as u64).with_style(pb_style());
    stg1.set_message("Cleaning data (stage 1)");
    let stg1 = points
        .par_iter()
        .progress_with(stg1)
        .map(|&p| {
            let rss = clean_point(&p, &point_map, config);
            RssRecord { point: p, rss }
        })
        .collect::<Vec<_>>();
    stg1
}

pub fn clean_records_stg2(raw_records: Vec<RssRecord>, config: &Config) -> Vec<RssRecord> {
    let point_map = PointMap::from_raw_records(raw_records);
    let points = point_map.all_points();
    let stg2 = indicatif::ProgressBar::new(points.len() as u64).with_style(pb_style());
    stg2.set_message("Cleaning data (stage 2) - itera  tion");
    let stg2 = points
        .par_iter()
        .progress_with(stg2)
        .map(|&p| {
            let rss = augment_point(&p, &point_map, config, config.augm_min_neighbors);
            RssRecord { point: p, rss }
        })
        .collect::<Vec<_>>();
    stg2
}

pub fn clean_led(point: Point, point_map: &PointMap<Vec<f32>>, config: &Config) -> Option<f32> {
    let neighbors = point_map
        .within_radius(point, config.clean_dist as usize)
        .into_iter()
        .flatten();
}

#[derive(Debug, Clone)]
pub struct RssScore {
    pub rss: f32,
    pub score: f32,
}

#[derive(Debug)]
pub struct CleanRecord {
    pub point: Point,
    pub rss: Vec<Option<RssScore>>,
}

impl CleanRecord {
    fn new(p: Point, config: &Config) -> Self {
        CleanRecord {
            point: p,
            rss: vec![None; config.led_count],
        }
    }

    fn update(&mut self, rss: &RssArr, scores: &[f32], config: &Config) {
        self.rss
            .iter_mut()
            .zip(rss)
            .zip(scores)
            .for_each(|((r, &rss), &score)| {
                if score > config.continuity_thresh {
                    return;
                }
                if let Some(old) = r {
                    if old.score > score {
                        *r = Some(RssScore { rss, score });
                    }
                } else {
                    *r = Some(RssScore { rss, score });
                }
            });
    }
}

fn clean_point(p: &Point, point_map: &PointMap, config: &Config) -> RssArr {
    let neighbors = point_map.rss_in_range(p, config.clean_dist);
    let continuity_scorer = ContinuityScorer::new(&neighbors, config);
    let candidates = point_map.rss_of(p);
    let clean_record = candidates
        .iter()
        .fold(CleanRecord::new(*p, config), |mut record, rss| {
            let scores = continuity_scorer.compute(rss);
            record.update(rss, &scores, config);
            record
        });
    clean_record
        .rss
        .into_iter()
        .map(|r| r.map_or(f32::NAN, |r| r.rss))
        .collect()
}

struct ContinuityScorer {
    neighbor_rss_avg: RssArr,
    darkness_penalty: f32,
}

impl ContinuityScorer {
    fn new(neighbors_rss: &[&RssArr], config: &Config) -> Self {
        let sums = neighbors_rss
            .iter()
            .fold(vec![0.0; config.led_count], |sum, rss| {
                sum.iter()
                    .zip(rss.iter())
                    .map(|(s, r)| s + r)
                    .collect::<RssArr>()
            });
        let avg = sums
            .into_iter()
            .map(|s| s / neighbors_rss.len() as f32)
            .collect();
        ContinuityScorer {
            neighbor_rss_avg: avg,
            darkness_penalty: config.darkness_penalty,
        }
    }

    fn compute(&self, rss: &RssArr) -> Vec<f32> {
        let f = |(val, avg)| {
            if val > avg {
                (val - avg) / self.darkness_penalty
            } else {
                avg - val
            }
        };
        rss.iter().zip(&self.neighbor_rss_avg).map(f).collect()
    }
}
