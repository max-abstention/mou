use {
    crate::{
        indexing::{index_statements, Index},
        query, Config, Statements,
    },
    anyhow::{Context, Result},
    async_trait::async_trait,
    std::sync::OnceLock,
    swiftide::{indexing::EmbeddedField, integrations::duckdb::Duckdb},
};

static DUCK_DB: OnceLock<Duckdb> = OnceLock::new();

/// Retrieves a static duckdb
///
/// # Panics
/// Panics if it cannot setup duckdb
pub fn get_duckdb(config: &Config) -> Duckdb {
    DUCK_DB
        .get_or_init(|| build_duckdb(config).expect("Failed to build duckdb (2)"))
        .to_owned()
}

// Probably should just be on the repository/config, cloned from there.
// This sucks in tests
pub(crate) fn build_duckdb(config: &Config) -> Result<Duckdb> {
    std::fs::create_dir_all(config.cache_dir())?;
    let path = config.cache_dir().join("duck.db3");

    tracing::debug!("Building Duckdb: {}", path.display());

    let embedding_provider = config.embedding_provider();

    let connection =
        duckdb::Connection::open(&path).context("Failed to open connection to duckdb")?;
    Duckdb::builder()
        .connection(connection)
        .with_vector(
            EmbeddedField::Combined,
            embedding_provider.vector_size().try_into()?,
        )
        .table_name(normalize_table_name(&config.project_name))
        .cache_table(format!(
            "cache_{}",
            normalize_table_name(&config.project_name)
        ))
        .build()
        .context("Failed to build Duckdb")
}

// Is this enough?
fn normalize_table_name(name: &str) -> String {
    name.replace('-', "_")
}
#[derive(Clone, Debug, Default)]
pub struct DuckdbIndex {}

impl DuckdbIndex {
    #[allow(clippy::unused_self)]
    fn get_duckdb(&self, config: &Config) -> Duckdb {
        get_duckdb(config)
    }
}

#[async_trait]
impl Index for DuckdbIndex {
    async fn query_statements(&self, config: &Config, query_: &str) -> Result<String> {
        let storage = self.get_duckdb(config);
        query::query(config, &storage, query_).await
    }

    async fn index_statements(&self, statements: Statements, config: &Config) -> Result<()> {
        let storage = self.get_duckdb(config);
        index_statements(statements, config, &storage).await
    }
}
