use serde::{Deserialize, Serialize};

/// 应用全局配置（本地 JSON 持久化，敏感信息本地保存，不写死代码）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default)]
    pub sftp: SftpConfig,
    #[serde(default)]
    pub genata: GenataConfig,
    #[serde(default)]
    pub tange: TangeConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            sftp: SftpConfig::default(),
            genata: GenataConfig::default(),
            tange: TangeConfig::default(),
        }
    }
}

/// SFTP 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpConfig {
    #[serde(default = "default_sftp_host")]
    pub host: String,
    #[serde(default = "default_sftp_port")]
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub remote_dir: String,
}

impl Default for SftpConfig {
    fn default() -> Self {
        Self {
            host: default_sftp_host(),
            port: default_sftp_port(),
            username: String::new(),
            password: String::new(),
            remote_dir: String::new(),
        }
    }
}

/// GenataTech 平台配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenataConfig {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_genata_base_url")]
    pub base_url: String,
    #[serde(default = "default_product_id")]
    pub product_id: String,
}

impl Default for GenataConfig {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            base_url: default_genata_base_url(),
            product_id: default_product_id(),
        }
    }
}

/// Tange 平台配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TangeConfig {
    #[serde(default)]
    pub app_id: String,
    #[serde(default)]
    pub access_key: String,
    #[serde(default)]
    pub access_secret: String,
    #[serde(default = "default_firmware_id")]
    pub firmware_id: String,
}

impl Default for TangeConfig {
    fn default() -> Self {
        Self {
            app_id: String::new(),
            access_key: String::new(),
            access_secret: String::new(),
            firmware_id: default_firmware_id(),
        }
    }
}

fn default_sftp_host() -> String {
    "www.genatatech.com".to_string()
}

fn default_sftp_port() -> u16 {
    22
}

fn default_genata_base_url() -> String {
    "https://www.genatatech.com".to_string()
}

fn default_product_id() -> String {
    "188".to_string()
}

fn default_firmware_id() -> String {
    "GENATA_JZ-T32LQ_IPC_Z300S-4G".to_string()
}

/**
 * 获取配置文件路径（config_dir/firmware-pusher/config.json）
 */
pub fn config_path() -> std::path::PathBuf {
    let dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    dir.join("firmware-pusher").join("config.json")
}

/**
 * 从磁盘加载配置，文件不存在或解析失败则返回默认值
 */
pub fn load() -> AppConfig {
    match std::fs::read_to_string(config_path()) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

/**
 * 保存配置到磁盘（自动创建目录）
 * @param {AppConfig} config - 待保存配置
 */
pub fn save(config: &AppConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {e}"))?;
    }
    let text = serde_json::to_string_pretty(config).map_err(|e| format!("序列化配置失败: {e}"))?;
    std::fs::write(&path, text).map_err(|e| format!("写入配置失败: {e}"))
}
