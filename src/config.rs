use serde::{Deserialize, Serialize};
use std::fs;

static mut CONFIG: Option<Config> = None;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Config {
    /// TSN 排程失敗
    pub w0: f64,
    /// AVB 排程失敗的數量
    pub w1: f64,
    /// AVB 的平均 Worst case delay
    pub w2: f64,
    /// 重排路徑的成本
    pub w3: f64,
    /// 快速終止模式，看見第一組可行解即返回
    pub fast_stop: bool,
    /// 計算能見度時，TSN 對舊路徑的偏好程度
    pub tsn_memory: f64,
    /// 計算能見度時，AVB 對舊路徑的偏好程度
    pub avb_memory: f64,
    /// 演算法最多能執行的時間，以微秒計
    pub t_limit: u128
}

impl Config {
    pub fn load() -> Self {
        let txt = fs::read_to_string("config.json")
            .unwrap_or_else(|_| fs::read_to_string("config.example.json").unwrap());
        let config: Config = serde_json::from_str(&txt).expect(&format!("無法設定檔"));
        config
    }
    pub fn get() -> &'static Self {
        unsafe {
            if CONFIG.is_none() {
                CONFIG = Some(Config::load());
            }
            CONFIG.as_ref().unwrap()
        }
    }
}
