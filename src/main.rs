use augment::AugmentBox;
use clap::Parser;
use indicatif::{ProgressBar, ProgressIterator};
use point::Point;
use std::{error::Error, io, path::PathBuf};

use clean::{clean_records_stg1, clean_records_stg2};
use config::Config;
use rss_record::RssRecord;

mod augment;
mod clean;
mod config;
mod point;
mod point_map;
mod rss_record;

#[derive(Debug, Parser)]
struct Cli {
    /// Input file
    input: PathBuf,

    /// Output file, prints to stdout if not present
    #[arg(short, long, value_name = "OUT_FILE")]
    output: Option<PathBuf>,

    /// Configuration file, uses default values for values not present
    #[arg(short, long, value_name = "CONFIG_FILE")]
    config: Option<PathBuf>,

    /// Specifies whether to clean the data
    #[arg(long)]
    clean: bool,

    /// Specifies the number of augmentation iterations to perform after cleaning
    #[arg(
        long = "cl-aug-iters",
        default_value_t = 1,
        value_name = "COUNT",
        requires = "clean"
    )]
    clean_augment_iters: u32,

    /// Specifies whether to augment the data
    #[arg(long)]
    augment: bool,
}

fn aug_boxes() -> Vec<AugmentBox> {
    vec![
        AugmentBox::new_with_size(Point::new(0, 0), 1210, 1210),
        AugmentBox::new_with_size(Point::new(1610, 0), 1210, 1210),
        AugmentBox::new_with_size(Point::new(0, 1550), 1210, 1210),
        AugmentBox::new_with_size(Point::new(1610, 1550), 1210, 1210),
    ]
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let config = if let Some(config_path) = cli.config {
        Config::from_file(&config_path)?
    } else {
        Config::default()
    };

    let mut rdr = csv::Reader::from_path(cli.input)?;

    let mut records = rdr
        .deserialize()
        .collect::<Result<Vec<RssRecord>, csv::Error>>()?;

    if cli.clean {
        records = clean_records_stg1(records, &config);
        let num_iters = cli.clean_augment_iters;
        let iter_pb = ProgressBar::new(num_iters as u64).with_style(config::pb_style2());
        iter_pb.set_message("Cleaning data (stage 2)");
        for _ in (0..cli.clean_augment_iters).progress_with(iter_pb) {
            records = clean_records_stg2(records, &config);
        }
    }

    if cli.augment {
        let mut point_map = point_map::PointMap::from_raw_records(records.clone());
        augment::populate_points(&mut point_map, &aug_boxes(), &config);
        let to_augment = point_map.all_points();
        let augment_pb = ProgressBar::new(to_augment.len() as u64).with_style(config::pb_style());
        augment_pb.set_message("Augmenting data");
        for it in 0..20 {
            augment_pb.reset();
            records = to_augment
                .iter()
                .progress_with(augment_pb.clone())
                .map(|p| RssRecord {
                    point: *p,
                    rss: augment::augment_point(p, &point_map, &config, config.augm_min_neighbors2),
                })
                .collect();
            point_map = point_map::PointMap::from_raw_records(records.clone());
        }
    }

    let output = cli.output.as_ref().map_or_else(
        || -> io::Result<Box<dyn std::io::Write>> { Ok(Box::new(std::io::stdout())) },
        |path| -> io::Result<Box<dyn std::io::Write>> {
            Ok(Box::new(std::fs::File::create(path)?))
        },
    )?;

    let mut wtr = csv::Writer::from_writer(output);
    let headers = ["x", "y"]
        .into_iter()
        .map(|s| s.to_owned())
        .chain((0..config.led_count).map(|i| format!("led_{}", i)));
    wtr.write_record(headers)?;

    for record in records {
        let mut row = vec![record.point.x.to_string(), record.point.y.to_string()];
        row.extend(record.rss.iter().map(|rss| rss.to_string()));
        wtr.write_record(row)?;
    }

    Ok(())
}
