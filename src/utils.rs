#[macro_export]
macro_rules! result_err {
    () => {
        |err| {
            info!("err = {}", err);
            format!("{:?}", err)
        }
    };
}
