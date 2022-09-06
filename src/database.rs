pub struct Database
{
    client: tokio_postgres::Client,
    get_statement: tokio_postgres::Statement,
    insert_statement: tokio_postgres::Statement,
}

impl Database
{
    pub async fn new(connect_string: &str) -> Result<Self, tokio_postgres::Error>
    {
        let (client, connection) = tokio_postgres::connect(connect_string, tokio_postgres::NoTls).await?;

        tokio::spawn(connection);

        let get_statement = client.prepare("SELECT hash FROM hashes WHERE index = $1").await?;
        let insert_statement = client.prepare("INSERT INTO hashes (index, hash) VALUES ($1, $2)").await?;

        Ok(Self {
            client,
            get_statement,
            insert_statement,
        })
    }

    pub async fn hashes_for_index(&self, index: i32) -> Result<Vec<String>, tokio_postgres::Error>
    {
        self.client.query(&self.get_statement, &[ &index ]).await?
                   .into_iter()
                   .map(|r| r.try_get(0))
                   .collect()
    }

    pub async fn add_hash(&self, index: i32, hash: &str) -> Result<(), tokio_postgres::Error>
    {
        self.client.execute(&self.insert_statement, &[ &index, &hash ]).await?;
        Ok(())
    }

    pub async fn metadata(&self, key: &str) -> Result<Option<String>, tokio_postgres::Error>
    {
        let row = self.client.query_opt("SELECT value FROM metadata WHERE key=$1", &[ &key ]).await?;

        Ok(match row {
            Some(r) => Some(r.get(0)),
            None => None
        })
    }

    pub async fn set_metadata(&self, key: &str, value: &str) -> Result<(), tokio_postgres::Error>
    {
        self.client.execute("INSERT INTO metadata (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value=$2", &[ &key, &value ]).await?;
        Ok(())
    }
}