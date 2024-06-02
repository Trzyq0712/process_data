use crate::{
    config::Config,
    point::Point,
    point_map::PointMap,
    rss_record::{RssArr, RssRecord},
};

pub struct AugmentBox {
    ll: Point,
    ur: Point,
    resolution: u32,
}

impl AugmentBox {
    pub fn new(ll: Point, ur: Point) -> Self {
        Self {
            ll,
            ur,
            resolution: 10,
        }
    }

    pub fn new_with_size(ll: Point, w: u32, h: u32) -> Self {
        Self {
            ll,
            ur: Point {
                x: ll.x + w,
                y: ll.y + h,
            },
            resolution: 10,
        }
    }
}

pub fn populate_points(point_map: &mut PointMap, boxes: &[AugmentBox], config: &Config) {
    for AugmentBox { ll, ur, resolution } in boxes {
        let x_range = (ll.x..ur.x).step_by(*resolution as usize);
        let y_range = (ll.y..ur.y).step_by(*resolution as usize);
        for x in x_range {
            for y in y_range.clone() {
                point_map.insert_if_absent(Point { x, y }, vec![f32::NAN; config.led_count]);
            }
        }
    }
}

pub fn augment_point(
    point: &Point,
    point_map: &PointMap,
    config: &Config,
    min_pts: usize,
) -> RssArr {
    let mut rss = if !point_map.contains(point) {
        return vec![f32::NAN; config.led_count];
    } else {
        let rss_vec = point_map.rss_of(point);
        assert_eq!(rss_vec.len(), 1);
        rss_vec[0].clone()
    };
    let neighbors = point_map.in_range(point, config.augm_dist);

    for i in 0..config.led_count {
        // If the RSS value is already computed, skip it
        if rss[i].is_finite() {
            continue;
        }
        let (aug_sum, aug_cnt) = neighbors
            .iter()
            .filter_map(|(p_src, rss_src)| {
                if rss_src[i].is_nan() {
                    None
                } else {
                    Some(compute_augmentation(rss_src[i], p_src, i, point, config))
                }
            })
            .fold((0.0, 0_usize), |(sum, count), rss| (sum + rss, count + 1));
        if aug_cnt < min_pts {
            continue;
        }

        rss[i] = aug_sum / aug_cnt as f32;
    }
    rss
}

pub fn augment_all_points(
    to_augment: &[Point],
    point_map: &PointMap,
    config: &Config,
) -> Vec<RssRecord> {
    to_augment
        .iter()
        .map(|p| RssRecord {
            point: *p,
            rss: augment_point(p, point_map, config, config.augm_min_neighbors),
        })
        .collect()
}

fn point_led_distance(&Point { x: x1, y: y1 }: &Point, led_idx: usize, config: &Config) -> f32 {
    let Point { x: x2, y: y2 } = config.led_positions[led_idx];
    let dx = x1 as f32 - x2 as f32;
    let dy = y1 as f32 - y2 as f32;
    let dz = config.height as f32;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn compute_augmentation(
    rss: f32,
    src: &Point,
    led_idx: usize,
    tgt: &Point,
    config: &Config,
) -> f32 {
    let d_src = point_led_distance(src, led_idx, config);
    let d_tgt = point_led_distance(tgt, led_idx, config);
    let a_src = (config.prop_loss_func)(d_src);
    let a_tgt = (config.prop_loss_func)(d_tgt);
    let cos_src = (config.height as f32) / d_src;
    let cos_tgt = (config.height as f32) / d_tgt;
    // If the target point is outside the LED's field of view, return 0
    // if cos_tgt.acos() > config.led_fov {
    //     return 0.0;
    // }
    rss * a_tgt / a_src * (cos_tgt / cos_src).powf(config.lambertian_order + 1.0)
}
