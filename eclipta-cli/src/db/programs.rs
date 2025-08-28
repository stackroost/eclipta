use sqlx::{Pool, Postgres};

#[derive(Debug)]
pub struct Program {
    pub id: i32,
    pub title: String,
    pub version: String,
    pub status: String,
    pub path: String,
}

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

pub async fn list_programs(pool: &Pool<Postgres>) -> Result<Vec<Program>, sqlx::Error> {
    let rows = sqlx::query_as!(
        Program,
        r#"
        SELECT 
            id, 
            title, 
            version, 
            status, 
            path
        FROM ebpf_programs
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn delete_program(pool: &Pool<Postgres>, program_id: i32) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM ebpf_programs WHERE id = $1",
        program_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_program_by_id(
    pool: &Pool<Postgres>,
    program_id: i32,
) -> Result<Option<Program>, sqlx::Error> {
    let row = sqlx::query_as!(
        Program,
        r#"
        SELECT 
            id, 
            title, 
            version, 
            status, 
            path
        FROM ebpf_programs
        WHERE id = $1
        "#,
        program_id
    )
    .fetch_optional(pool)  
    .await?;

    Ok(row)
}

pub async fn get_program_by_title(
    pool: &Pool<Postgres>,
    title: &str,
) -> Result<Vec<Program>, sqlx::Error> {
    let rows = sqlx::query_as!(
        Program,
        r#"
        SELECT 
            id, 
            title, 
            version, 
            status, 
            path
        FROM ebpf_programs
        WHERE title = $1
        ORDER BY created_at DESC
        "#,
        title
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

