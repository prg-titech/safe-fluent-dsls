use std::fmt::{Display, Formatter};
use nonempty::NonEmpty;

type Selection = String;

type Table = String;

type Predicate = String;

pub struct Query {
    selections: Vec<Selection>,
    tables: Vec<Table>,
    where_clauses: Vec<Predicate>
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "SELECT {}\nFROM {}{}",
            self.selections.join(", "),
            self.tables.join(" "),
            self.where_clauses.iter()
                   .map(|p| "\nWHERE ".to_string() + p)
                   .collect::<String>()
        )
    }
}

pub struct QueryBuilder<S, T, W> {
    selections: S,
    tables: T,
    where_clauses: W
}

type EmptyQuery = QueryBuilder<Vec<Selection>, Vec<Table>, Vec<Predicate>>;

type PartialQuery = QueryBuilder<NonEmpty<Selection>, Vec<Table>, Vec<Predicate>>;

type PartialQuery2 = QueryBuilder<NonEmpty<Selection>, NonEmpty<Table>, Vec<Predicate>>;

type CompleteQuery = QueryBuilder<NonEmpty<Selection>, NonEmpty<Table>, NonEmpty<Predicate>>;

impl Query {
    pub fn builder() -> EmptyQuery {
        EmptyQuery::new()
    }
}

impl EmptyQuery {
    pub fn new() -> Self {
        Self {
            selections: vec![],
            tables: vec![],
            where_clauses: vec![]
        }
    }

    pub fn select<S: ToString>(self, selection: S) -> PartialQuery {
        PartialQuery {
            selections: NonEmpty::new(selection.to_string()),
            tables: self.tables,
            where_clauses: self.where_clauses
        }
    }
}

impl PartialQuery {
    pub fn select<S: ToString>(mut self, selection: S) -> PartialQuery {
        self.selections.push(selection.to_string());
        self
    }

    pub fn from<S: ToString>(self, table: S) -> PartialQuery2 {
        PartialQuery2 {
            selections: self.selections,
            tables: NonEmpty::new(table.to_string()),
            where_clauses: self.where_clauses
        }
    }
}

impl PartialQuery2 {
    pub fn from<S: ToString>(mut self, table: S) -> PartialQuery2 {
        self.tables.push(table.to_string());
        self
    }

    pub fn where_<S: ToString>(self, predicate: S) -> CompleteQuery {
        CompleteQuery {
            selections: self.selections,
            tables: self.tables,
            where_clauses: NonEmpty::new(predicate.to_string())
        }
    }

    pub fn build(self) -> Query {
        Query {
            selections: self.selections.into(),
            tables: self.tables.into(),
            where_clauses: self.where_clauses
        }
    }
}

impl CompleteQuery {
    pub fn where_<S: ToString>(mut self, predicate: S) -> CompleteQuery {
        self.where_clauses.push(predicate.to_string());
        self
    }

    pub fn build(self) -> Query {
        Query {
            selections: self.selections.into(),
            tables: self.tables.into(),
            where_clauses: self.where_clauses.into()
        }
    }
}