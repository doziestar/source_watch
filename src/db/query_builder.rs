use serde_json::Value;
use std::collections::HashMap;
use diesel::RunQueryDsl;
use thiserror::Error;

// Type-safe field and table names
macro_rules! define_type_safe_names {
    ($name:ident, $($variant:ident),*) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant),*,
            Custom(String),
        }

        impl $name {
            pub fn as_str(&self) -> &str {
                match self {
                    $($name::$variant => stringify!($variant)),*,
                    $name::Custom(s) => s,
                }
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                match s {
                    $(stringify!($variant) => $name::$variant),*,
                    _ => $name::Custom(s.to_string()),
                }
            }
        }
    };
}

define_type_safe_names!(Field, Id, Name, Email, Age, CreatedAt, UpdatedAt);
define_type_safe_names!(Table, Users, Posts, Comments, Products, Orders);

#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseType {
    PostgreSQL,
    MongoDB,
    Redis,
    Cassandra,
    Elasticsearch,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryOperation {
    Select,
    Insert,
    Update,
    Delete,
    Aggregate,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub(crate) field: Field,
    pub(crate) operator: Operator,
    pub(crate) value: Value,
}

#[derive(Debug, Clone)]
pub enum Operator {
    Eq,
    Ne,
    Gt,
    Lt,
    Gte,
    Lte,
    Like,
    In,
    NotIn,
}

#[derive(Debug, Clone)]
pub struct OrderBy {
    pub(crate) field: Field,
    pub(crate) direction: OrderDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Error, Debug)]
pub enum QueryBuilderError {
    #[error("Invalid operation for database type")]
    InvalidOperation,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Unsupported database type")]
    UnsupportedDatabaseType,
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

pub struct QueryBuilder {
    pub database_type: DatabaseType,
    pub operation: QueryOperation,
    pub table: Table,
    pub fields: Vec<Field>,
    pub conditions: Vec<Condition>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub values: Vec<Value>,
}

impl QueryBuilder {
    pub fn new(database_type: DatabaseType) -> Self {
        QueryBuilder {
            database_type,
            operation: QueryOperation::Select,
            table: Table::Custom("".to_string()),
            fields: Vec::new(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            values: Vec::new(),
        }
    }

    pub fn table(mut self, table: Table) -> Self {
        self.table = table;
        self
    }

    pub fn operation(mut self, operation: QueryOperation) -> Self {
        self.operation = operation;
        self
    }

    pub fn field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    pub fn condition(mut self, field: Field, operator: Operator, value: Value) -> Self {
        self.conditions.push(Condition { field, operator, value });
        self
    }

    pub fn order_by(mut self, field: Field, direction: OrderDirection) -> Self {
        self.order_by.push(OrderBy { field, direction });
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn value(mut self, value: Value) -> Self {
        self.values.push(value);
        self
    }

    pub fn build(&self) -> Result<String, QueryBuilderError> {
        match self.database_type {
            DatabaseType::PostgreSQL => postgresql::build_query(self),
            DatabaseType::MongoDB => mongodb::build_query(self),
            DatabaseType::Redis => redis::build_query(self),
            DatabaseType::Cassandra => cassandra::build_query(self),
            DatabaseType::Elasticsearch => elasticsearch::build_query(self),
        }
    }
}

pub struct DatabaseManager {
    pools: HashMap<DatabaseType, Box<dyn DatabasePool>>,
}

impl DatabaseManager {
    pub fn new() -> Self {
        DatabaseManager {
            pools: HashMap::new(),
        }
    }

    pub fn add_pool(&mut self, db_type: DatabaseType, pool: Box<dyn DatabasePool>) {
        self.pools.insert(db_type, pool);
    }

    pub async fn execute(&self, query_builder: &QueryBuilder) -> Result<Vec<HashMap<String, Value>>, QueryBuilderError> {
        let pool = self.pools.get(&query_builder.database_type)
            .ok_or_else(|| QueryBuilderError::UnsupportedDatabaseType)?;
        let query = query_builder.build()?;
        pool.execute(&query, query_builder.values.clone()).await
    }
}