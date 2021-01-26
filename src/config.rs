use std::sync::Arc;
use std::sync::Mutex;

use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;
#[derive(Clone, Debug)]
pub struct Config {
    pub ip: String,
}

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn init_config() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    let _ = RUNTIME.set(Runtime::new().unwrap()).unwrap();
}

pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get().unwrap()
}

impl Config {
    pub fn get_instance() -> Arc<Mutex<Config>> {
        static mut CONFIG: Option<Arc<Mutex<Config>>> = None;

        unsafe {
            // Rust中使用可变静态变量都是unsafe的
            CONFIG
                .get_or_insert_with(|| {
                    init_config();
                    // 初始化单例对象的代码
                    Arc::new(Mutex::new(Config {
                        ip: whoami::hostname(),
                    }))
                })
                .clone()
        }
    }

    pub fn set_ip(&mut self, ip: &str) {
        self.ip = ip.to_string();
    }

    pub fn ip() -> String {
        Config::get_instance().lock().unwrap().ip.clone()
    }
}
