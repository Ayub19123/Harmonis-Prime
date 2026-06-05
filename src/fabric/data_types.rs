use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

/// ScalarValue: The atomic unit of data in the fabric
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScalarValue {
    Null,
    Boolean(bool),
    Int64(i64),
    Float64(f64),
    Utf8(String),
    Binary(Vec<u8>),
    Timestamp(i64),
    Duration(i64),
}

/// Series: A typed, contiguous array
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Series {
    pub name: String,
    pub data: Vec<ScalarValue>,
    pub dtype: DataType,
}

/// DataType: The type system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Null,
    Boolean,
    Int64,
    Float64,
    Utf8,
    Binary,
    Timestamp,
    Duration,
    List(Box<DataType>),
    Struct(Vec<(String, DataType)>),
}

/// Schema: The type of a DataFrame
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub dtype: DataType,
    pub nullable: bool,
}

/// Timestamp: A point in time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub i64);

/// Duration: A time interval
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Duration(pub i64);

impl Add<Duration> for Timestamp {
    type Output = Timestamp;
    fn add(self, rhs: Duration) -> Self::Output {
        Timestamp(self.0 + rhs.0)
    }
}

impl Sub<Duration> for Timestamp {
    type Output = Timestamp;
    fn sub(self, rhs: Duration) -> Self::Output {
        Timestamp(self.0 - rhs.0)
    }
}

impl Sub<Timestamp> for Timestamp {
    type Output = Duration;
    fn sub(self, rhs: Timestamp) -> Self::Output {
        Duration(self.0 - rhs.0)
    }
}

impl Series {
    pub fn new(name: &str, data: Vec<ScalarValue>, dtype: DataType) -> Self {
        Self {
            name: name.to_string(),
            data,
            dtype,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&ScalarValue> {
        self.data.get(index)
    }

    pub fn map<F>(&self, f: F) -> Series
    where
        F: Fn(&ScalarValue) -> ScalarValue,
    {
        let new_data: Vec<ScalarValue> = self.data.iter().map(f).collect();
        Series {
            name: format!("{}_mapped", self.name),
            data: new_data,
            dtype: self.dtype.clone(),
        }
    }

    pub fn filter<F>(&self, predicate: F) -> Series
    where
        F: Fn(&ScalarValue) -> bool,
    {
        let filtered: Vec<ScalarValue> =
            self.data.iter().filter(|x| predicate(x)).cloned().collect();
        Series {
            name: format!("{}_filtered", self.name),
            data: filtered,
            dtype: self.dtype.clone(),
        }
    }
}

impl Schema {
    pub fn empty() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn with_field(mut self, name: &str, dtype: DataType, nullable: bool) -> Self {
        self.fields.push(Field {
            name: name.to_string(),
            dtype,
            nullable,
        });
        self
    }

    pub fn get_field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }

    pub fn project(&self, field_names: &[String]) -> Self {
        let projected: Vec<Field> = self
            .fields
            .iter()
            .filter(|f| field_names.contains(&f.name))
            .cloned()
            .collect();
        Self { fields: projected }
    }
}
