use reqwest::{Client, RequestBuilder};
use serde_json::{json, Value};

use crate::config::TangeConfig;

/// Tange 查询/推送接口域名（示例环境）
const TANGE_BASE: &str = "https://openapidemo.tange365.com/api/ota";

/**
 * 校验响应 code == 200，失败时返回 msg
 * @param {Value} json - 响应体
 */
fn check_ok(json: &Value) -> Result<(), String> {
    let code = json["code"].as_i64().unwrap_or(-1);
    if code == 200 {
        Ok(())
    } else {
        let msg = json["msg"].as_str().unwrap_or("未知错误");
        // 常见业务错误码翻译成友好提示
        let friendly = match msg {
            "firmware_version_referenced_by_rule" => "该固件版本仍被推送规则引用，请先取消推送规则再删除",
            _ => msg,
        };
        Err(format!("Tange 接口返回失败: {friendly}"))
    }
}

/**
 * 发送请求并解析 JSON 响应（失败时带上状态码与原始响应体，便于排查）
 * @param {RequestBuilder} req - 已构造的请求
 * @param {String} label - 操作名称
 * 返回解析后的 JSON
 */
async fn send_and_parse(req: RequestBuilder, label: &str) -> Result<Value, String> {
    let resp = req
        .send()
        .await
        .map_err(|e| format!("{label}请求失败: {e}"))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("{label}读取响应失败: {e}"))?;
    match serde_json::from_str::<Value>(&text) {
        Ok(json) => {
            check_ok(&json)?;
            Ok(json)
        }
        Err(e) => {
            // 截断原始响应，避免刷屏
            let snippet: String = text.chars().take(500).collect();
            Err(format!("{label}返回非 JSON（HTTP {status}）: {e}\n原始响应: {snippet}"))
        }
    }
}

/**
 * 查询已登记的固件版本列表，验证上传是否成功
 * @param {TangeConfig} cfg - Tange 配置
 * 返回完整响应 JSON
 */
pub async fn list_versions(cfg: &TangeConfig) -> Result<Value, String> {
    let client = Client::new();
    let url = format!("{TANGE_BASE}/firmwareVersionList");
    let req = client.post(&url).form(&[
        ("appid", cfg.app_id.as_str()),
        ("accessKey", cfg.access_key.as_str()),
        ("accessSecret", cfg.access_secret.as_str()),
        ("firmwareId", cfg.firmware_id.as_str()),
    ]);
    send_and_parse(req, "查询 Tange 版本列表").await
}

/**
 * 推送升级规则到指定设备
 * @param {TangeConfig} cfg - Tange 配置
 * @param {String} version_number - 版本号（如 1.0.0.5，不带 V）
 * @param {String} reason - 升级原因
 * @param {Vec<String>} device_ids - 设备 ID 列表
 * 返回完整响应 JSON
 */
pub async fn add_ota_rule(
    cfg: &TangeConfig,
    version_number: &str,
    reason: &str,
    device_ids: Vec<String>,
) -> Result<Value, String> {
    let client = Client::new();
    let url = format!("{TANGE_BASE}/addOtaRule");
    let body = json!({
        "appid": &cfg.app_id,
        "accessKey": &cfg.access_key,
        "accessSecret": &cfg.access_secret,
        "firmwareId": &cfg.firmware_id,
        "versionNumber": version_number,
        "reason": reason,
        "deviceIds": device_ids,
    });
    send_and_parse(client.post(&url).json(&body), "推送升级规则").await
}

/**
 * 登记固件版本到 Tange（demo 环境，用 appid/accessKey/accessSecret 认证，无需 HMAC 签名）
 * @param {TangeConfig} cfg - Tange 配置
 * @param {String} firmware_id - 固件 ID
 * @param {String} version_number - 版本号（不带 V）
 * @param {String} version_type - 版本类型（testing/release）
 * @param {String} download_link - CDN 下载地址
 * @param {String} md5 - 文件 MD5
 * @param {String} content - 版本说明
 * @param {i64} importance - 重要性（1/2/3）
 */
pub async fn publish_version(
    cfg: &TangeConfig,
    firmware_id: &str,
    version_number: &str,
    version_type: &str,
    download_link: &str,
    md5: &str,
    content: &str,
    importance: i64,
) -> Result<Value, String> {
    let client = Client::new();
    let url = format!("{TANGE_BASE}/addFirmwareVersion");
    let body = json!({
        "appid": &cfg.app_id,
        "accessKey": &cfg.access_key,
        "accessSecret": &cfg.access_secret,
        "firmwareId": firmware_id,
        "versionNumber": version_number,
        "versionType": version_type,
        "downloadLink": download_link,
        "md5": md5,
        "content": content,
        "importance": importance,
    });
    send_and_parse(client.post(&url).json(&body), "登记固件版本").await
}

/**
 * 取消设备升级规则
 * @param {TangeConfig} cfg - Tange 配置
 * @param {String} reason - 取消理由
 * @param {Vec<String>} device_ids - 设备 ID 列表
 */
pub async fn remove_ota_rule(
    cfg: &TangeConfig,
    reason: &str,
    device_ids: Vec<String>,
) -> Result<Value, String> {
    let client = Client::new();
    let url = format!("{TANGE_BASE}/removeOtaRule");
    let body = json!({
        "appid": &cfg.app_id,
        "accessKey": &cfg.access_key,
        "accessSecret": &cfg.access_secret,
        "firmwareId": &cfg.firmware_id,
        "deviceIds": device_ids,
        "reason": reason,
    });
    send_and_parse(client.post(&url).json(&body), "取消推送").await
}

/**
 * 删除固件版本
 * @param {TangeConfig} cfg - Tange 配置
 * @param {String} version_number - 版本号（不带 V）
 */
pub async fn delete_version(
    cfg: &TangeConfig,
    version_number: &str,
) -> Result<Value, String> {
    let client = Client::new();
    let url = format!("{TANGE_BASE}/delFirmwareVersion");
    let body = json!({
        "appid": &cfg.app_id,
        "accessKey": &cfg.access_key,
        "accessSecret": &cfg.access_secret,
        "firmwareId": &cfg.firmware_id,
        "versionNumber": version_number,
    });
    send_and_parse(client.post(&url).json(&body), "删除固件版本").await
}
