use crate::{point::Point, rss_record::RssRecord};

#[derive(Debug, Clone)]
pub struct PointMapConfig {
    pub x_size: usize,
    pub y_size: usize,
    pub resolution: usize,
}

#[derive(Debug, Clone)]
pub struct PointMap<T> {
    data: Vec<T>,
    conf: PointMapConfig,
}

fn extract_points(raw_records: &[RssRecord]) -> Vec<Point> {
    let mut points = raw_records.iter().map(|r| r.point).collect::<Vec<_>>();
    points.sort_unstable();
    points.dedup();
    points
}

impl PointMap<Vec<f32>> {
    pub fn from_raw_records(
        records: impl IntoIterator<Item = (Point, f32)>,
        config: PointMapConfig,
    ) -> Self {
        let mut map = Self::new(config);
        for (p, rss) in records {
            map.add_record(p, rss);
        }
        map
    }

    pub fn add_record(&mut self, p: Point, rss: f32) {
        self[p].push(rss);
    }
}

impl PointMap<Option<f32>> {
    pub fn missing_points(&self) -> Vec<Point> {
        let mut res = Vec::new();
        for (idx, rss) in self.data.iter().enumerate() {
            if rss.is_none() {
                res.push(Point::new(
                    idx % self.conf.x_size * self.conf.resolution,
                    idx / self.conf.x_size * self.conf.resolution,
                ));
            }
        }
        res
    }

    pub fn to_records(&self) -> Vec<(Point, f32)> {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(idx, rss)| {
                rss.map(|rss| {
                    (
                        Point::new(
                            idx % self.conf.x_size * self.conf.resolution,
                            idx / self.conf.x_size * self.conf.resolution,
                        ),
                        rss,
                    )
                })
            })
            .collect()
    }

    pub fn subsample(&self, mut point_gen: impl FnMut() -> Point, mut count: usize) -> Self {
        let mut res = Self::new(self.conf.clone());
        while count > 0 {
            let p = point_gen();
            if let Some(rss) = self[p] {
                if res[p].is_none() {
                    res[p] = Some(rss);
                    count -= 1;
                }
            }
        }
        res
    }
}

impl<T> PointMap<T> {
    fn get_index(&self, p: Point) -> usize {
        p.y as usize * self.conf.x_size + p.x as usize
    }

    fn ranges(&self, p: Point, r: usize) -> (usize, usize, usize, usize) {
        let x0 = p.x.saturating_sub(r) / self.conf.resolution;
        let x1 = ((p.x + r) / self.conf.resolution).min(self.conf.x_size);
        let y0 = p.y.saturating_sub(r) / self.conf.resolution;
        let y1 = ((p.y + r) / self.conf.resolution).min(self.conf.y_size);
        (x0, x1, y0, y1)
    }

    pub fn within_radius(&self, p: Point, r: usize) -> Vec<&T> {
        let mut res = Vec::new();
        let (x0, x1, y0, y1) = self.ranges(p, r);
        for y in y0..y1 {
            for x in x0..x1 {
                let p2 = Point::new(x * self.conf.resolution, y * self.conf.resolution);
                if p.dist_sq(&p2) <= r * r {
                    res.push(&self[p2]);
                }
            }
        }
        res
    }

    pub fn within_square(&self, p: Point, r: usize) -> Vec<&T> {
        let mut res = Vec::new();
        let (x0, x1, y0, y1) = self.ranges(p, r);
        for y in y0..y1 {
            for x in x0..x1 {
                res.push(&self.data[y * self.conf.x_size + x]);
            }
        }
        res
    }
}

impl<T: Default> PointMap<T> {
    pub fn new(config: PointMapConfig) -> Self {
        let data = std::iter::repeat_with(T::default)
            .take(config.x_size * config.y_size)
            .collect();
        let x = config.x_size / config.resolution;
        let y = config.y_size / config.resolution;
        PointMap {
            data,
            conf: PointMapConfig {
                x_size: x,
                y_size: y,
                resolution: config.resolution,
            },
        }
    }
}

impl<T> std::ops::Index<Point> for PointMap<T> {
    type Output = T;

    fn index(&self, p: Point) -> &Self::Output {
        &self.data[self.get_index(p)]
    }
}

impl<T> std::ops::IndexMut<Point> for PointMap<T> {
    fn index_mut(&mut self, p: Point) -> &mut Self::Output {
        &mut self.data[self.get_index(p)]
    }
}
