use std::convert::TryInto;

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

pub async fn count(sql: &str) -> Result<u64, String> {
    let conn = get_instance().clone();
    let (count,): (i64,) = sqlx::query_as(sql)
        .fetch_one(&conn)
        .await
        .map_err(result_err!())?;

    info!("COUNT = {}", count);
    Ok(count.try_into().unwrap())
}

pub async fn execute(sql: &str) -> Result<(), String> {
    let conn = get_instance().clone();

    let _ = sqlx::query(sql)
        .execute(&conn)
        .await
        .map_err(result_err!())?;

    Ok(())
}

#[macro_export]
macro_rules! mysql_find_one {
    ($x:ty, $s:expr) => {{
        let conn = crate::mysql::get_instance().clone();
        let result = sqlx::query_as($s).fetch_one(&conn).await;
        match result {
            Ok(row) => {
                let (value,): ($x,) = row;
                Ok(value)
            }
            Err(err) => {
                info!("err = {}", err);
                let e = format!("{:?}", err);
                Err(e)
            }
        }
    }};
}

pub fn sql_page_str(sql: &str, limit: u32, page: u32) -> Result<String, String> {
    if limit < 1 || page < 1 {
        return Err("请确保每页大小和页数都大于".to_string());
    }
    Ok(format!(
        "{} limit {} offset {}",
        sql,
        limit,
        ((page - 1) * limit)
    ))
}

#[macro_export]
macro_rules! mysql_query {
    ($x:ty, $v:ident, $s:expr) => {{
        let conn = crate::mysql::get_instance().clone();
        let result = sqlx::query_as::<_, $x>($s).fetch_all(&conn).await;
        match result {
            Ok(list) => {
                let vec = list.to_vec();
                for x in &vec {
                    $v.push(x.clone())
                }

                Ok(())
            }
            Err(err) => {
                info!("err = {}", err);
                let e = format!("{:?}", err);
                Err(e)
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::User;
    use crate::{api::project::Project, config};
    use log::info;

    const SQL_PROJECT_COUNT: &'static str = "SELECT COUNT(project_id) FROM tb_project 
 where is_delete is null  or  is_delete != 'Y' and name is not null";

    async fn init() {
        config::init_config();
        // 数据库初始化
        let _ = crate::mysql::init(crate::mysql::URL).await;
    }

    #[actix_rt::test]
    async fn test_mysql_find_one() {
        init().await;

        let sql = format!(
            "select name from sys_user where username = '{}'",
            "sunmh@justsafe.com"
        );

        let result = mysql_find_one!(String, &sql);

        info!("result = {:?}", result);
    }

    #[actix_rt::test]
    async fn test_mysql() {
        init().await;

        let conn = super::get_instance().clone();

        let row: User = sqlx::query_as::<_, User>("SELECT * FROM sys_user WHERE username = ?")
            .bind("sunmh@justsafe.com")
            .fetch_one(&conn)
            .await
            .unwrap();

        info!("row = {:?}", row.password);
        // assert!(.is_ok());
    }

    #[actix_rt::test]
    async fn test_mysql_project_query() {
        init().await;

        let mut data: Vec<Project> = Vec::new();

        let msg = r#"    
        select project_id, no, name, status, create_time, create_user, update_time, update_user, version_svn_url
        from tb_project where is_delete is null  or  is_delete != 'Y' and name is not null
        order by project_id desc limit 20 offset 1
                "#;

        let _ = mysql_query!(Project, data, msg);

        // assert!(super::project_query(10, 1, &mut data).await.is_ok());
        info!("data = {}", serde_json::to_string_pretty(&data).unwrap());
        // assert!(super::project_query(10, 2, &mut data).await.is_ok());
        // assert!(super::project_query(20, 0, &mut data).await.is_err());
    }

    #[actix_rt::test]
    async fn test_mysql_count() {
        init().await;
        assert!(super::count(SQL_PROJECT_COUNT).await.is_ok());
    }

    #[actix_rt::test]
    async fn test_mysql_execute() {
        init().await;

        let sql = format!(
            r#"insert into tb_project (no, name, status, create_user, version_svn_url) 
            values ("test", "test", 1, "test", null)"#,
        );

        assert!(super::execute(&sql).await.is_ok());

        let sql2 = "UPDATE tb_project SET name = 'test2' where  name='test'";
        assert!(super::execute(&sql2).await.is_ok());

        let sql3 = "DELETE FROM tb_project where no = 'test' and name ='test2'";
        assert!(super::execute(&sql3).await.is_ok());
    }
}
