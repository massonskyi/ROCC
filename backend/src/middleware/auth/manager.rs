use chrono:: prelude::*;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;
use super::{models::UserModel, utils::PasswordManager};
use tokio_postgres::{Client, Error, NoTls};
use tokio::{self, sync::Mutex};

#[derive(utoipa::ToSchema)]
pub struct UserManager {
    pool: Arc<Mutex<Client>>,
}

impl Clone for UserManager {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }

}
impl UserManager {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        Ok(UserManager {
            pool: Arc::new(Mutex::new(client)),
        })
    }

    pub async fn create_user(
        &self,
        name: String,
        surname: String,
        age: i32,
        username: String,
        email: String,
        password: String,
        role: String,
        avatar: String,
    ) -> Result<UserModel, String> {
        let hash_password = PasswordManager::hash_password(&password).unwrap();
        let now = Utc::now().to_string();
        let user = UserModel {
            id: 0,
            name: name.clone(),
            surname: surname.clone(),
            age,
            username: username.clone(),
            email: email.clone(),
            hash_password: hash_password.clone(),
            created_at: now.clone(),
            updated_at: now.clone(),
            deleted_at: None,
            last_active: now.clone(),
            role: role.clone(),
            avatar: avatar.clone(),
            status: String::from("active"),
            token: Uuid::new_v4().to_string(),
        };

        let query = "
            INSERT INTO users (name, surname, age, username, email, hash_password, created_at, updated_at, last_active, role, avatar, status, token)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id
        ";

        let client = self.pool.lock().await;
        let stmt = client.prepare(&query).await.unwrap();
        
        let row = client
        .query_one(
            &stmt,
            &[
                &name, &surname, &age.to_string(), &username, &email, &hash_password, &now, &now,&now, &role,
                &avatar, &user.status, &user.token

            ],
        )
        .await.map_err(|e| format!("Failed to execute query: {}", e))?;

        let id: i32 = row.get(0);
        Ok(UserModel { id, ..user })
    }

    pub async fn update_user(
        &self,
        id: i32,
        update_data: HashMap<String, String>,
    ) -> Result<UserModel, String> {
        let mut user = self.get_user(id).await.ok_or("User not found")?;

        for (key, value) in update_data.iter() {
            match key.as_str() {
                "name" => user.name = value.clone(),
                "surname" => user.surname = value.clone(),
                "age" => user.age = value.parse().unwrap_or(user.age),
                "username" => user.username = value.clone(),
                "email" => user.email = value.clone(),
                "hash_password" => {
                    user.hash_password = PasswordManager::hash_password(value).unwrap()
                }
                "role" => user.role = value.clone(),
                "avatar" => user.avatar = value.clone(),
                "status" => user.status = value.clone(),
                _ => {}
            }
        }

        user.updated_at = Utc::now().to_string();
        let query = "
            UPDATE users
            SET name = $1, surname = $2, age = $3, username = $4, email = $5, hash_password = $6, updated_at = $7,last_active = $8 role = $9, avatar = $10, status = $11
            WHERE id = $14
        ";

        let client = self.pool.lock().await;
        let stmt = client.prepare(&query).await.unwrap();

        client
            .execute(
                &stmt,
                &[
                    &user.name, &user.surname, &user.age, &user.username, &user.email,
                    &user.hash_password, &user.updated_at, &user.role, &user.avatar,
                    &user.status, &user.last_active,
                    &id,
                ],
            )
            .await
            .unwrap();

        Ok(user)
    }

    pub async fn delete_user(&self, id: i32) -> Result<(), String> {
        let _user = self.get_user(id).await.ok_or("User not found")?;

        let query = "UPDATE users SET deleted_at = $1 WHERE id = $2";
        let client = self.pool.lock().await;
        let stmt = client.prepare(query).await.unwrap();

        client
            .execute(&stmt, &[&Utc::now().to_string(), &id.to_string()])
            .await
            .unwrap();

        Ok(())
    }


    pub async fn get_user(&self, id: i32) -> Option<UserModel> {
        let query = "
            SELECT * FROM users
            WHERE id = $1
        ";

        let client = self.pool.lock().await;
        let stmt = client.prepare(query).await.unwrap();

        if let Ok(row) = client.query_one(&stmt, &[&id]).await {
            Some(UserModel {
                id: row.get(0),
                name: row.get(1),
                surname: row.get(2),
                age: row.get(3),
                username: row.get(4),
                email: row.get(5),
                hash_password: row.get(6),
                created_at: row.get(7),
                updated_at: row.get(8),
                deleted_at: row.get(9),
                last_active: row.get(10),
                role: row.get(11),
                avatar: row.get(12),
                status: row.get(13),
                token: row.get(14),
            })
        } else {
            None
        }
    }

    pub async fn list_users(&self) -> Vec<UserModel> {
        let query = "
            SELECT *
            FROM users
        ";

        let client = self.pool.lock().await;
        let stmt = client.prepare(query).await.unwrap();
        let rows = client.query(&stmt, &[]).await.unwrap();

        rows.iter()
            .map(|row| UserModel {
                id: row.get(0),
                name: row.get(1),
                surname: row.get(2),
                age: row.get(3),
                username: row.get(4),
                email: row.get(5),
                hash_password: row.get(6),
                created_at: row.get(7),
                updated_at: row.get(8),
                deleted_at: row.get(9),
                last_active: row.get(10),
                role: row.get(11),
                avatar: row.get(12),
                status: row.get(13),
                token: row.get(14),
            }).collect()
        }

        pub async fn get_user_by_username(&self, username: &str) -> Result<Option<UserModel>, Error> {
            let query = "
                SELECT *
                FROM users
                WHERE username = $1
            ";
            let client = self.pool.lock().await;
            let stmt = client.prepare(query).await?;
            let row = client.query_opt(&stmt, &[&username]).await?;
            
            match row {
                Some(row) => {
                    let id: i32 = row.try_get(0)?;
                    let name: String = row.try_get(1)?;
                    let surname: String = row.try_get(2)?;
                    let age: i32 = row.try_get(3)?;
                    let username: String = row.try_get(4)?;
                    let email: String = row.try_get(5)?;
                    let hash_password: String = row.try_get(6)?;
                    let created_at: String = row.try_get(7)?;
                    let updated_at: String = row.try_get(8)?;
                    let deleted_at: Option<String> = row.try_get(9)?;
                    let last_active: String = row.try_get(10)?;
                    let role: String = row.try_get(11)?;
                    let avatar: String = row.try_get(12)?;
                    let status: String = row.try_get(13)?;
                    let token: String = row.try_get(14)?;

                        
                    // Convert JSON fields
                    let user = UserModel {
                        id,
                        name,
                        surname,
                        age,
                        username,
                        email,
                        hash_password,
                        created_at,
                        updated_at,
                        deleted_at,
                        last_active,
                        role,
                        avatar,
                        status,
                        token,
                    };
    
                    Ok(Some(user))
                },
                None => Ok(None),
            }
        }
}