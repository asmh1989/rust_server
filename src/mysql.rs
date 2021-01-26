use once_cell::sync::OnceCell;
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

use crate::{result_err, sha::sha256_encode};
use log::info;

static INSTANCE: OnceCell<Pool<MySql>> = OnceCell::new();

#[derive(sqlx::FromRow, Debug)]
pub struct User {
    username: String,
    password: String,
    salt: String,
}

pub static URL: &'static str = "mysql://androidversion:androidversion@192.168.10.63:3306/androidversion?allowMultiQueries=true&useUnicode=true&characterEncoding=UTF-8";

pub fn get_instance() -> &'static Pool<MySql> {
    INSTANCE.get().expect("mysql need init first")
}

pub async fn init(url: &str) -> Result<(), sqlx::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await?;

    INSTANCE.set(pool).expect("mysql init error");
    info!("mysql init success!");
    Ok(())
}

pub async fn login(username: &str, password: &str) -> Result<(), String> {
    let conn = get_instance().clone();
    let row: User = sqlx::query_as::<_, User>("SELECT * FROM sys_user WHERE username = ?")
        .bind(username)
        .fetch_one(&conn)
        .await
        .map_err(result_err!())?;

    let pd = sha256_encode(password, &row.salt);

    if pd == row.password {
        Ok(())
    } else {
        Err("用户名或密码错误".to_string())
    }
}

#[cfg(test)]
mod tests {
    use log::info;

    use crate::config;

    use super::User;

    #[actix_rt::test]
    async fn test_mysql() {
        config::init_config();

        // 数据库初始化
        let _ = crate::mysql::init(crate::mysql::URL).await;

        let conn = super::get_instance().clone();

        let row: User = sqlx::query_as::<_, User>("SELECT * FROM sys_user WHERE username = ?")
            .bind("sunmh@justsafe.com")
            .fetch_one(&conn)
            .await
            .unwrap();

        info!("row = {:?}", row.password);
        // assert!(.is_ok());
    }
}
