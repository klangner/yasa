// Read CSV file aith the output from the `hackrf_sweep`, `soapy_power`, or `rtl_power` output.

use chrono::{NaiveDate, NaiveTime};
use csv::ReaderBuilder;
use serde::Deserialize;


pub struct DataFrame {
    records: Vec<CsvRecord>,
    freq_low: u64,
    freq_high: u64,
    freq_step: f32,
    sweep_steps: usize,
}

#[derive(Debug, Deserialize)]
pub struct CsvRecord {
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
    pub fn from_string(data: &str) -> Self {
        let mut rdr = ReaderBuilder::new()
            .has_headers(false)
            .trim(csv::Trim::All)
            .from_reader(data.as_bytes());
    
        let records: Vec<CsvRecord> = rdr.deserialize().flatten().collect();

        let default = CsvRecord::default();
        let first = records.first().unwrap_or(&default);
        let sweep_steps = records.iter()
            .skip(1)
            .take_while(|r| r.freq_low > first.freq_low)
            .count() + 1;
        let last = records.get(sweep_steps - 1).unwrap_or(&default);
        let freq_low = first.freq_low;
        let freq_high = last.freq_high;
        let freq_step = first.freq_step;
        
        Self {
            records,
            freq_low,
            freq_high,
            freq_step,
            sweep_steps,
        }
    }
}

impl Default for CsvRecord {
    fn default() -> Self {
        Self { 
            date: Default::default(), 
            time: Default::default(), 
            freq_low: Default::default(), 
            freq_high: Default::default(), 
            freq_step: Default::default(), 
            num_samples: Default::default(), 
            samples: Default::default() }
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
        let df = DataFrame::from_string(&csv);

        assert_eq!(df.freq_low, 144_000_000);
        assert_eq!(df.freq_high, 146_000_000);
        assert_eq!(df.freq_step, 976.56);
        assert_eq!(df.sweep_steps, 2);
        assert_eq!(df.records.len(), 4);
    }
}