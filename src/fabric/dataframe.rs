use crate::fabric::data_types::{ScalarValue, Schema, Series};
use serde::{Deserialize, Serialize};

/// DataFrame: A typed, columnar table
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataFrame {
    pub name: String,
    pub schema: Schema,
    pub columns: Vec<Series>,
    pub row_count: usize,
}

pub type Column = Series;

#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

impl DataFrame {
    pub fn empty(name: &str) -> Self {
        Self {
            name: name.to_string(),
            schema: Schema::empty(),
            columns: Vec::new(),
            row_count: 0,
        }
    }

    pub fn from_columns(name: &str, columns: Vec<Series>, schema: Schema) -> Result<Self, String> {
        let lengths: Vec<usize> = columns.iter().map(|c| c.len()).collect();
        if lengths.is_empty() {
            return Ok(Self::empty(name));
        }
        let first_len = lengths[0];
        if !lengths.iter().all(|&l| l == first_len) {
            return Err("Column length mismatch".to_string());
        }

        Ok(Self {
            name: name.to_string(),
            schema,
            columns,
            row_count: first_len,
        })
    }

    pub fn select(&self, col_names: &[String]) -> Result<DataFrame, String> {
        let selected_cols: Vec<Series> = col_names
            .iter()
            .filter_map(|name| self.columns.iter().find(|c| &c.name == name).cloned())
            .collect();

        if selected_cols.len() != col_names.len() {
            return Err("Some columns not found in schema".to_string());
        }

        let projected_schema = self.schema.project(col_names);

        Ok(DataFrame {
            name: format!("{}_select", self.name),
            schema: projected_schema,
            columns: selected_cols,
            row_count: self.row_count,
        })
    }

    pub fn filter<F>(&self, predicate: F) -> DataFrame
    where
        F: Fn(&[ScalarValue]) -> bool,
    {
        let mut filtered_columns: Vec<Vec<ScalarValue>> =
            self.columns.iter().map(|_| Vec::new()).collect();

        for row_idx in 0..self.row_count {
            let row: Vec<ScalarValue> = self
                .columns
                .iter()
                .map(|col| col.get(row_idx).unwrap_or(&ScalarValue::Null).clone())
                .collect();

            if predicate(&row) {
                for (col_idx, col) in self.columns.iter().enumerate() {
                    filtered_columns[col_idx]
                        .push(col.get(row_idx).unwrap_or(&ScalarValue::Null).clone());
                }
            }
        }

        let new_columns: Vec<Series> = self
            .columns
            .iter()
            .enumerate()
            .map(|(idx, col)| Series {
                name: col.name.clone(),
                data: filtered_columns[idx].clone(),
                dtype: col.dtype.clone(),
            })
            .collect();

        DataFrame {
            name: format!("{}_filter", self.name),
            schema: self.schema.clone(),
            columns: new_columns,
            row_count: filtered_columns.get(0).map(|c| c.len()).unwrap_or(0),
        }
    }

    pub fn group_by(&self, _group_cols: &[String], _agg_cols: &[String]) -> DataFrame {
        let mut result = DataFrame::empty(&format!("{}_groupby", self.name));
        result.schema = self.schema.clone();
        result
    }

    pub fn join(
        &self,
        other: &DataFrame,
        _left_col: &str,
        _right_col: &str,
        _how: JoinType,
    ) -> Result<DataFrame, String> {
        let mut result = DataFrame::empty(&format!("{}_join_{}", self.name, other.name));
        result.schema = self.schema.clone();
        Ok(result)
    }

    pub fn get_row(&self, index: usize) -> Option<Vec<ScalarValue>> {
        if index >= self.row_count {
            return None;
        }
        Some(
            self.columns
                .iter()
                .map(|col| col.get(index).unwrap_or(&ScalarValue::Null).clone())
                .collect(),
        )
    }

    pub fn len(&self) -> usize {
        self.row_count
    }

    pub fn is_empty(&self) -> bool {
        self.row_count == 0
    }
}
