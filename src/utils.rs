#[macro_export]
macro_rules! result_err {
    () => {
        |err| {
            info!("err = {}", err);
            format!("{:?}", err)
        }
    };
}

#[macro_export]
macro_rules! response_auth_err {
    ($s:expr) => {
        response_error2(serde_json::json!({
            "code": 401,
            "msg": $s
        }))
    };
}
