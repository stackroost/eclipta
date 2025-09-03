use sqlx::{Pool, Postgres, Row};

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
    let rows = sqlx::query(
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

    let programs = rows
        .iter()
        .map(|row| Program {
            id: row.get("id"),
            title: row.get("title"),
            version: row.get("version"),
            status: row.get("status"),
            path: row.get("path"),
        })
        .collect();

    Ok(programs)
}

pub async fn delete_program(pool: &Pool<Postgres>, program_id: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM ebpf_programs WHERE id = $1"
    )
    .bind(program_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_program_by_id(
    pool: &Pool<Postgres>,
    program_id: i32,
) -> Result<Option<Program>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT 
            id, 
            title, 
            version, 
            status, 
            path
        FROM ebpf_programs
        WHERE id = $1
        "#
    )
    .bind(program_id)
    .fetch_optional(pool)
    .await?;

    let program = row.map(|row| Program {
        id: row.get("id"),
        title: row.get("title"),
        version: row.get("version"),
        status: row.get("status"),
        path: row.get("path"),
    });

    Ok(program)
}

pub async fn get_program_by_title(
    pool: &Pool<Postgres>,
    title: &str,
) -> Result<Vec<Program>, sqlx::Error> {
    let rows = sqlx::query(
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
        "#
    )
    .bind(title)
    .fetch_all(pool)
    .await?;

    let programs = rows
        .iter()
        .map(|row| Program {
            id: row.get("id"),
            title: row.get("title"),
            version: row.get("version"),
            status: row.get("status"),
            path: row.get("path"),
        })
        .collect();

    Ok(programs)
}

