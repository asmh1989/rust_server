use once_cell::sync::OnceCell;
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

static INSTANCE: OnceCell<Pool<MySql>> = OnceCell::new();

pub fn get_instance() -> &'static Pool<MySql> {
    INSTANCE.get().expect("mysql need init first")
}

pub async fn init(url: &str) -> Result<(), sqlx::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await?;

    INSTANCE.set(pool).expect("mysql init error");

    Ok(())
}

#[cfg(test)]
mod tests {
    use log::info;

    #[derive(sqlx::FromRow, Debug)]
    struct User {
        username: String,
        password: String,
        salt: String,
    }

    static URL: &'static str = "mysql://androidversion:androidversion@192.168.10.63:3306/androidversion?allowMultiQueries=true&useUnicode=true&characterEncoding=UTF-8";
    #[actix_rt::test]
    async fn test_mysql() {
        crate::config::Config::get_instance();

        let _ = super::init(URL).await;
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
