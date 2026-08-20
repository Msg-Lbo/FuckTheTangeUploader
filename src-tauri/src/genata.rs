use std::sync::{Mutex, OnceLock};

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::GenataConfig;

/// access token 内存缓存（应用运行期间复用，重启后重新登录）
static TOKEN_CACHE: OnceLock<Mutex<Option<String>>> = OnceLock::new();

/**
 * 版本列表项（响应 data.results 元素）
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionItem {
    pub id: i64,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub next_version: Option<String>,
    #[serde(default)]
    pub join_time: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub file_size: Option<i64>,
    #[serde(default)]
    pub md5: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/**
 * 校验响应 code == 1，失败时返回 msg
 * @param {serde_json::Value} json - 响应体
 */
fn check_code(json: &serde_json::Value) -> Result<(), String> {
    let code = json["code"].as_i64().unwrap_or(-1);
    if code == 1 {
        Ok(())
    } else {
        let msg = json["msg"].as_str().unwrap_or("未知错误").to_string();
        Err(format!("接口返回失败: {msg}"))
    }
}

/**
 * 登录获取 access token
 * @param {GenataConfig} cfg - 平台配置
 * 返回 access token 字符串
 */
pub async fn login(cfg: &GenataConfig) -> Result<String, String> {
    let client = Client::new();
    let url = format!("{}/api/user/auth/token/obtain", cfg.base_url);
    let params = [
        ("username", cfg.username.as_str()),
        ("password", cfg.password.as_str()),
        ("pwd_type", "password"),
    ];
    let resp = client
        .post(&url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("登录请求失败: {e}"))?;
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析登录响应失败: {e}"))?;
    check_code(&json)?;
    let token = json["data"]["access"]
        .as_str()
        .ok_or("登录响应缺少 access token")?
        .to_string();
    Ok(token)
}

/**
 * 获取 access token（优先读缓存，未登录则登录）
 * @param {GenataConfig} cfg - 平台配置
 */
pub async fn get_token(cfg: &GenataConfig) -> Result<String, String> {
    let cache = TOKEN_CACHE.get_or_init(|| Mutex::new(None));
    if let Some(token) = cache.lock().unwrap().clone() {
        return Ok(token);
    }
    let token = login(cfg).await?;
    *cache.lock().unwrap() = Some(token.clone());
    Ok(token)
}

/**
 * 查询服务器上已存在的固件文件名列表
 * @param {GenataConfig} cfg - 平台配置
 * 返回文件名数组
 */
pub async fn list_files(cfg: &GenataConfig) -> Result<Vec<String>, String> {
    let token = get_token(cfg).await?;
    let client = Client::new();
    let url = format!("{}/api/product/firmware/file/list", cfg.base_url);
    let json: serde_json::Value = client
        .post(&url)
        .bearer_auth(&token)
        .form(&[
            ("page", "1"),
            ("size", "999"),
            ("product_id", cfg.product_id.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("查询固件文件列表失败: {e}"))?
        .json()
        .await
        .map_err(|e| format!("解析固件文件列表失败: {e}"))?;
    check_code(&json)?;
    let results = json["data"]["results"]
        .as_array()
        .ok_or("响应缺少 data.results")?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    Ok(results)
}

/**
 * 查询版本列表（含 id/version/md5 等，取最新一条 id 作为后续 p_id）
 * @param {GenataConfig} cfg - 平台配置
 */
pub async fn list_versions(cfg: &GenataConfig) -> Result<Vec<VersionItem>, String> {
    let token = get_token(cfg).await?;
    let client = Client::new();
    let url = format!("{}/api/product/firmware/version/list", cfg.base_url);
    let mut page = 1usize; // 当前页码
    let page_size = 10usize; // 接口约定的每页数量
    let mut versions = Vec::new(); // 完整版本列表

    loop {
        let page_text = page.to_string(); // 表单页码
        let page_size_text = page_size.to_string(); // 表单每页数量
        let json: serde_json::Value = client
            .post(&url)
            .bearer_auth(&token)
            .query(&[
                ("page", page_text.as_str()),
                ("size", page_size_text.as_str()),
            ])
            .form(&[("product_id", cfg.product_id.as_str())])
            .send()
            .await
            .map_err(|e| format!("查询版本列表第 {page} 页失败: {e}"))?
            .json()
            .await
            .map_err(|e| format!("解析版本列表第 {page} 页失败: {e}"))?;
        check_code(&json)?;
        let total = json["data"]["total"]
            .as_u64()
            .ok_or_else(|| format!("版本列表第 {page} 页响应缺少 data.total"))?
            as usize; // 服务端版本总数
        let page_versions: Vec<VersionItem> =
            serde_json::from_value(json["data"]["results"].clone())
                .map_err(|e| format!("解析版本列表第 {page} 页结构失败: {e}"))?;
        let page_count = page_versions.len(); // 当前页实际数量
        versions.extend(page_versions);

        if versions.len() >= total {
            break;
        }
        if page_count == 0 {
            return Err(format!(
                "版本列表分页异常: 已获取 {} 条，服务端声明共 {total} 条",
                versions.len()
            ));
        }
        page += 1;
    }

    Ok(versions)
}

/**
 * 登记新版本（后端据此完成 CDN 上传）
 * @param {GenataConfig} cfg - 平台配置
 * @param {String} version - 版本号（带 V 前缀，如 V1.0.0.5）
 * @param {String} filename - 文件名
 * @param {i64} p_id - 上一步版本列表最新一条的 id
 * @param {String} description - 版本描述
 * 返回新版本 id
 */
pub async fn insert_version(
    cfg: &GenataConfig,
    version: &str,
    filename: &str,
    p_id: i64,
    description: &str,
) -> Result<i64, String> {
    let token = get_token(cfg).await?;
    let client = Client::new();
    let url = format!("{}/api/product/firmware/version/insert", cfg.base_url);
    let p_id_str = p_id.to_string();
    let params = [
        ("product_id", cfg.product_id.as_str()),
        ("version", version),
        ("filename", filename),
        ("p_id", p_id_str.as_str()),
        ("description", description),
    ];
    let json: serde_json::Value = client
        .post(&url)
        .bearer_auth(&token)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("登记版本失败: {e}"))?
        .json()
        .await
        .map_err(|e| format!("解析登记版本响应失败: {e}"))?;
    check_code(&json)?;
    let id = json["data"]["id"].as_i64().ok_or("响应缺少 data.id")?;
    Ok(id)
}
