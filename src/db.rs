use dashmap::DashMap;
use rand::{prelude::IteratorRandom, thread_rng};
use sqlx::{migrate::Migrator, SqlitePool};

use std::path::Path;

use crate::champions::CHAMPIONS;

pub struct Database {
    pool: SqlitePool,
    cache: DashMap<i64, (String, i64)>,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        Ok(Self {
            pool: SqlitePool::connect(url).await?,
            cache: DashMap::new(),
        })
    }

    pub async fn run_migrations(&self, path: &Path) -> Result<(), sqlx::Error> {
        let migrator = Migrator::new(path).await?;

        migrator.run(&self.pool).await?;

        Ok(())
    }

    pub async fn get_champion_and_rate(&self, user_id: i64) -> Result<(String, i64), sqlx::Error> {
        if let Some(entry) = self.cache.get(&user_id) {
            return Ok(entry.clone());
        }

        let row = sqlx::query!("SELECT champion, rate FROM users WHERE id=?;", user_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok((row.champion, row.rate))
        } else {
            let champion = {
                let mut rng = thread_rng();
                CHAMPIONS.keys().choose(&mut rng).unwrap()
            };

            sqlx::query!(
                "INSERT INTO users (id, champion, rate) VALUES (?, ?, ?);",
                user_id,
                champion,
                10
            )
            .execute(&self.pool)
            .await?;

            Ok((champion.clone(), 10))
        }
    }

    pub async fn set_champion(&self, user_id: i64, champion: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO users (id, champion, rate) VALUES (?, ?, ?) ON CONFLICT(id) DO UPDATE SET champion=?;",
            user_id,
            champion,
            10,
            champion
        )
        .execute(&self.pool)
        .await?;

        self.cache.remove(&user_id);

        Ok(())
    }

    pub async fn set_rate(&self, user_id: i64, rate: i64) -> Result<(), sqlx::Error> {
        let champion = {
            let mut rng = thread_rng();
            CHAMPIONS.keys().choose(&mut rng).unwrap()
        };

        sqlx::query!(
            "INSERT INTO users (id, champion, rate) VALUES (?, ?, ?) ON CONFLICT(id) DO UPDATE SET rate=?;",
            user_id,
            champion,
            rate,
            rate
        )
        .execute(&self.pool)
        .await?;

        self.cache.remove(&user_id);

        Ok(())
    }
}
