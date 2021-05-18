use super::errors::DataError;
use super::{Database, Variables};
use async_trait::async_trait;
use std::collections::HashMap;

struct UserVariableRow {
    key: String,
    value: i32,
}

#[async_trait]
impl Variables for Database {
    async fn get_user_variables(
        &self,
        user: &str,
        room_id: &str,
    ) -> Result<HashMap<String, i32>, DataError> {
        let rows = sqlx::query!(
            r#"SELECT key, value as "value: i32" FROM user_variables
               WHERE room_id = ? AND user_id = ?"#,
            room_id,
            user
        )
        .fetch_all(&self.conn)
        .await?;

        Ok(rows.into_iter().map(|row| (row.key, row.value)).collect())
    }

    async fn get_variable_count(&self, user: &str, room_id: &str) -> Result<i32, DataError> {
        let row = sqlx::query!(
            r#"SELECT count(*) as "count: i32" FROM user_variables
               WHERE room_id = ? and user_id = ?"#,
            room_id,
            user
        )
        .fetch_optional(&self.conn)
        .await?;

        Ok(row.map(|r| r.count).unwrap_or(0))
    }

    async fn get_user_variable(
        &self,
        user: &str,
        room_id: &str,
        variable_name: &str,
    ) -> Result<i32, DataError> {
        let row = sqlx::query!(
            r#"SELECT value as "value: i32" FROM user_variables
               WHERE user_id = ? AND room_id = ? AND key = ?"#,
            user,
            room_id,
            variable_name
        )
        .fetch_one(&self.conn)
        .await?;
        // .map_err(|e| match e {
        //     sqlx::Error::RowNotFound => Err(DataError::KeyDoesNotExist(variable_name.clone())),
        //     _ => Err(e.into()),
        // })?;

        Ok(row.value)
    }

    async fn set_user_variable(
        &self,
        user: &str,
        room_id: &str,
        variable_name: &str,
        value: i32,
    ) -> Result<(), DataError> {
        sqlx::query(
            "INSERT INTO user_variables
                    (user_id, room_id, key, value)
                    values (?, ?, ?, ?)",
        )
        .bind(user)
        .bind(room_id)
        .bind(variable_name)
        .bind(value)
        .execute(&self.conn)
        .await?;

        Ok(())
    }

    async fn delete_user_variable(
        &self,
        user: &str,
        room_id: &str,
        variable_name: &str,
    ) -> Result<(), DataError> {
        sqlx::query(
            "DELETE FROM user_variables
             WHERE user_id = ? AND room_id = ? AND variable_name = ?",
        )
        .bind(user)
        .bind(room_id)
        .bind(variable_name)
        .execute(&self.conn)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_db() -> Database {
        let db_path = tempfile::NamedTempFile::new_in(".").unwrap();
        crate::db::sqlite::migrator::migrate(db_path.path().to_str().unwrap())
            .await
            .unwrap();

        Database::new(db_path.path().to_str().unwrap())
            .await
            .unwrap()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn set_and_get_variable_test() {
        use super::super::Variables;
        let db = create_db().await;

        db.set_user_variable("myuser", "myroom", "myvariable", 1)
            .await
            .expect("Could not set variable");

        let value = db
            .get_user_variable("myuser", "myroom", "myvariable")
            .await
            .expect("Could not get variable");

        assert_eq!(value, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn get_missing_variable_test() {
        use super::super::Variables;
        let db = create_db().await;

        let value = db.get_user_variable("myuser", "myroom", "myvariable").await;

        println!("{:?}", value);
        assert!(value.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn get_other_user_variable_test() {
        use super::super::Variables;
        let db = create_db().await;

        db.set_user_variable("myuser1", "myroom", "myvariable", 1)
            .await
            .expect("Could not set variable");

        let value = db
            .get_user_variable("myuser2", "myroom", "myvariable")
            .await;

        println!("{:?}", value);
        assert!(value.is_err());
    }
}
