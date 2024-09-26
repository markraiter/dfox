use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableSchema {
    pub table_name: String,
    pub columns: Vec<ColumnSchema>,
    pub indexes: Vec<IndexSchema>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColumnSchema {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexSchema {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
}
