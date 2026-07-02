use std::{str::FromStr};

use chrono::{DateTime, Duration, Utc};
use machine_core::PropertySchema;
use serde::{Deserialize, de};

#[derive(Clone, Copy)]
pub enum Table {
    Float,
    Integer,
}

impl Table {
    pub fn from_property_spec(spec: &PropertySchema) -> Self {
        use PropertySchema::*;

        match spec {
            Integer { .. } | Boolean => Table::Integer,
            Float { .. } | UoM { .. } => Table::Float,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Table::Float => "float",
            Table::Integer =>"integer",
        }
    }
}

pub struct TimeSpan {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

impl TimeSpan {
    pub fn new(from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>) -> Self {
        Self { from, to }
    }
}

#[derive(Clone, Default)]
pub struct Interval {
    seconds: u64,
    minutes: u64,
    hours: u64,
    days: u64,
    week: u64,
}

impl Interval {
    pub fn to_duration(&self) -> Duration {
        let mut seconds: i64 = 0;

        seconds += self.seconds as i64;
        seconds += (self.minutes as i64) * 60;
        seconds += (self.hours as i64) * 3600;
        seconds += (self.days as i64) * 86_400;
        seconds += (self.week as i64) * 7 * 86_400;

        Duration::seconds(seconds)
    }

    pub fn to_ch(&self) -> String {
        let mut parts = Vec::new();

        if self.week > 0 {
            parts.push(format!("INTERVAL {} WEEK", self.week));
        }
        if self.days > 0 {
            parts.push(format!("INTERVAL {} DAY", self.days));
        }
        if self.hours > 0 {
            parts.push(format!("INTERVAL {} HOUR", self.hours));
        }
        if self.minutes > 0 {
            parts.push(format!("INTERVAL {} MINUTE", self.minutes));
        }
        if self.seconds > 0 {
            parts.push(format!("INTERVAL {} SECOND", self.seconds));
        }

        if parts.is_empty() {
            return "INTERVAL 0 SECOND".to_string();
        }

        parts.join(" + ")
    }
}

impl FromStr for Interval {
    type Err = String;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let mut out = Interval::default();

        while !s.is_empty() {
            let start = s
                .chars()
                .position(|c| !c.is_ascii_digit())
                .ok_or_else(|| format!("missing unit in interval: {s}"))?;

            if start == 0 {
                return Err(format!("expected number, got: {s}"));
            }

            let (num_str, rest) = s.split_at(start);

            let value: u64 = num_str
                .parse()
                .map_err(|_| format!("invalid number: {num_str}"))?;

            let mut chars = rest.chars();
            let unit = chars
                .next()
                .ok_or_else(|| format!("missing unit after: {num_str}"))?;

            s = chars.as_str();

            match unit {
                's' => out.seconds += value,
                'm' => out.minutes += value,
                'h' => out.hours += value,
                'd' => out.days += value,
                'w' => out.week += value,
                _ => return Err(format!("invalid unit: {unit}")),
            }
        }

        Ok(out)
    }
}

impl<'de> Deserialize<'de> for Interval {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Interval::from_str(&s).map_err(de::Error::custom)
    }
}

// ordering
#[derive(Clone, Copy, Deserialize)]
pub enum Ordering {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

impl Ordering {
    pub fn to_ch(self) -> &'static str {
        use Ordering::*;
        match self {
            Ascending => "ASC",
            Descending => "DESC",
        }
    }
}

// format
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    Json,
    ApacheArrow,
}

// aggregation
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AggregationOperation {
    Min,
    Max,
    #[serde(rename = "avg")]
    Average,
    Median,
    Sum,
    Count,
    First,
    Last,
}

impl AggregationOperation {
    pub fn to_ch(self) -> &'static str {
        use AggregationOperation::*;
        match self {
            Average => "avg(value)",
            Median => "median(value)",
            Min => "min(value)",
            Max => "max(value)",
            Sum => "sum(value)",
            Count => "count()",
            First => "argMin(value, ts)",
            Last => "argMax(value, ts)",
        }
    }
}

#[derive(Clone)]
pub struct Aggregation {
    pub operation: AggregationOperation,
    pub interval: Interval,
}