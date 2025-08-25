
use sqlx::{Pool, Postgres};


pub async fn insert_program(
    pool: &Pool<Postgres>,
    title: &str,
    description: &str,
    version: &str,
    path: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO ebpf_programs (title, description, version, status, path)
        VALUES ($1, $2, $3, 'deactive', $4)
        "#,
    )
    .bind(title)
    .bind(description)
    .bind(version)
    .bind(path)
    .execute(pool)
    .await?;

    Ok(())
}