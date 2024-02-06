// Read CSV file aith the output from the `hackrf_sweep`, `soapy_power`, or `rtl_power` output.

use std::error::Error;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use csv::ReaderBuilder;
use serde::Deserialize;


pub struct DataFrame {
    timestamps: Vec<NaiveDateTime>,
    freq_low: u64,
    freq_high: u64,
    freq_step: f32,
    num_bins: usize,
    data: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct Record {
    #[serde(with = "custom_date")]
    pub date: NaiveDate,
    #[serde(with = "custom_time")]
    pub time: NaiveTime,
    pub freq_low: u64,
    pub freq_high: u64,
    pub freq_step: f32,
    pub num_samples: u32,
    pub samples: Vec<f32>,
}

mod custom_date {
    use chrono::naive::NaiveDate;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Ok(NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap())
    }
}

mod custom_time {
    use chrono::naive::NaiveTime;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Ok(NaiveTime::parse_from_str(&s, "%H:%M:%S%.f").unwrap())
    }
}

impl DataFrame {
    pub fn from_string(data: &str) -> Result<Self, Box<dyn Error>> {
        let mut rdr = ReaderBuilder::new()
            .has_headers(false)
            .trim(csv::Trim::All)
            .from_reader(data.as_bytes());
    
        let mut timestamps = vec![];
        let mut freq_low = u64::MAX;
        let mut freq_high = u64::MIN;
        let mut step: Option<f32> = None;
        let mut data = vec![];
        for result in rdr.deserialize() {
            // Break out on error and try to continue processing, for cases where the CSV is still being written
            if let Err(e) = result {
                println!("Warning: {}", e);
                break;
            }
            let mut record: Record = result?;
            freq_low = std::cmp::min(freq_low, record.freq_low);
            freq_high = std::cmp::max(freq_high, record.freq_high);
            if let Some(s) = step {
                if (s - record.freq_step).abs() > std::f32::EPSILON {
                    return Err("Frequency step must be constant".into());
                }
            } else {
                step = Some(record.freq_step);
            }

            let ts = NaiveDateTime::new(record.date, record.time);
            if timestamps.last().map(|l| *l < ts).unwrap_or(true) {
                timestamps.push(ts);
            }

            data.append(&mut record.samples);
        }
        
        let num_bins = if timestamps.len() > 0 {data.len() / timestamps.len()} else {0};
        Ok(Self {
            timestamps,
            freq_low,
            freq_high,
            freq_step: step.unwrap_or(0.),
            num_bins,
            data,
        })
    }
}

/// ------------------------------------------------------------------------------------------------
/// Module unit tests
/// ------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_csv() {
        let csv = "\
            2024-02-03, 14:11:38, 144000000, 145000000, 976.56, 2, -10.0, -11.0
            2024-02-03, 14:11:48, 145000000, 146000000, 976.56, 2, -20.0, -21.0
            2024-02-03, 14:12:38, 144000000, 145000000, 976.56, 2, -30.0, -31.0
            2024-02-03, 14:12:48, 145000000, 146000000, 976.56, 2, -40.0, -41.0
        ";
        let df = DataFrame::from_string(&csv).unwrap();

        assert_eq!(df.freq_low, 144_000_000);
        assert_eq!(df.freq_high, 146_000_000);
        assert_eq!(df.freq_step, 976.56);
        assert_eq!(df.num_bins, 4);
        assert_eq!(df.data.len(), 8);
    }
}