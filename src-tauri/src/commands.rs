use serde_json::Value;

use crate::config::{AppConfig, GenataConfig, SftpConfig, TangeConfig};
use crate::genata::VersionItem;

/**
 * 加载配置
 * 返回应用配置
 */
#[tauri::command]
pub async fn load_config() -> Result<AppConfig, String> {
    Ok(crate::config::load())
}

/**
 * 保存配置到本地
 * @param {AppConfig} config - 应用配置
 */
#[tauri::command]
pub async fn save_config(config: AppConfig) -> Result<(), String> {
    crate::config::save(&config)
}

/**
 * SFTP 上传 Bin 文件并返回 MD5
 * @param {String} local_path - 本地文件路径
 * @param {String} remote_filename - 远程文件名
 * @param {SftpConfig} sftp_cfg - SFTP 配置
 * 返回文件 MD5（小写 hex）
 */
#[tauri::command]
pub async fn upload_sftp(
    local_path: String,
    remote_filename: String,
    sftp_cfg: SftpConfig,
) -> Result<String, String> {
    // 计算 MD5（分块读，放到阻塞线程池避免卡 UI）
    let md5 = {
        let p = local_path.clone();
        tokio::task::spawn_blocking(move || crate::sftp::md5_of_file(&p))
            .await
            .map_err(|e| format!("计算 MD5 失败: {e}"))??
    };

    // SFTP 上传（纯 Rust async，直接 await）
    crate::sftp::upload(&local_path, &remote_filename, &sftp_cfg).await?;

    Ok(md5)
}

/**
 * 列出 SFTP 远程目录文件
 * @param {SftpConfig} sftp_cfg - SFTP 配置
 * @param {String} remote_dir - 远程目录
 */
#[tauri::command]
pub async fn sftp_list_files(
    sftp_cfg: SftpConfig,
    remote_dir: String,
) -> Result<Vec<crate::sftp::SftpFileEntry>, String> {
    crate::sftp::list_files(&sftp_cfg, &remote_dir).await
}

/**
 * GenataTech 登录
 * @param {GenataConfig} genata_cfg - 平台配置
 */
#[tauri::command]
pub async fn genata_login(genata_cfg: GenataConfig) -> Result<String, String> {
    crate::genata::login(&genata_cfg).await
}

/**
 * GenataTech 查询已存在固件文件列表
 * @param {GenataConfig} genata_cfg - 平台配置
 */
#[tauri::command]
pub async fn genata_list_files(genata_cfg: GenataConfig) -> Result<Vec<String>, String> {
    crate::genata::list_files(&genata_cfg).await
}

/**
 * GenataTech 查询版本列表
 * @param {GenataConfig} genata_cfg - 平台配置
 */
#[tauri::command]
pub async fn genata_list_versions(genata_cfg: GenataConfig) -> Result<Vec<VersionItem>, String> {
    crate::genata::list_versions(&genata_cfg).await
}

/**
 * GenataTech 登记新版本
 * @param {GenataConfig} genata_cfg - 平台配置
 * @param {String} version - 版本号（带 V 前缀）
 * @param {String} filename - 文件名
 * @param {i64} p_id - 上一步最新版本 id
 * @param {String} description - 描述
 */
#[tauri::command]
pub async fn genata_insert_version(
    genata_cfg: GenataConfig,
    version: String,
    filename: String,
    p_id: i64,
    description: String,
) -> Result<i64, String> {
    crate::genata::insert_version(&genata_cfg, &version, &filename, p_id, &description).await
}

/**
 * Tange 查询固件版本列表
 * @param {TangeConfig} tange_cfg - Tange 配置
 */
#[tauri::command]
pub async fn tange_list_versions(tange_cfg: TangeConfig) -> Result<Value, String> {
    crate::tange::list_versions(&tange_cfg).await
}

/**
 * Tange 推送升级规则
 * @param {TangeConfig} tange_cfg - Tange 配置
 * @param {String} version_number - 版本号（不带 V）
 * @param {String} reason - 升级原因
 * @param {Vec<String>} device_ids - 设备 ID 列表
 */
#[tauri::command]
pub async fn tange_add_rule(
    tange_cfg: TangeConfig,
    version_number: String,
    reason: String,
    device_ids: Vec<String>,
) -> Result<Value, String> {
    crate::tange::add_ota_rule(&tange_cfg, &version_number, &reason, device_ids).await
}

/**
 * Tange 取消设备升级规则
 * @param {TangeConfig} tange_cfg - Tange 配置
 * @param {String} reason - 取消理由
 * @param {Vec<String>} device_ids - 设备 ID 列表
 */
#[tauri::command]
pub async fn tange_remove_rule(
    tange_cfg: TangeConfig,
    reason: String,
    device_ids: Vec<String>,
) -> Result<Value, String> {
    crate::tange::remove_ota_rule(&tange_cfg, &reason, device_ids).await
}

/**
 * Tange 删除固件版本
 * @param {TangeConfig} tange_cfg - Tange 配置
 * @param {String} version_number - 版本号（不带 V）
 */
#[tauri::command]
pub async fn tange_delete_version(
    tange_cfg: TangeConfig,
    version_number: String,
) -> Result<Value, String> {
    crate::tange::delete_version(&tange_cfg, &version_number).await
}

/**
 * Tange 登记固件版本（需 HMAC 签名，暂未实现）
 * @param {TangeConfig} tange_cfg - Tange 配置
 * @param {String} firmware_id - 固件 ID
 * @param {String} version_number - 版本号（不带 V）
 * @param {String} version_type - 版本类型（testing/release）
 * @param {String} download_link - CDN 下载地址
 * @param {String} md5 - 文件 MD5
 * @param {String} content - 版本说明
 * @param {i64} importance - 重要性（1/2/3）
 */
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn tange_publish_version(
    tange_cfg: TangeConfig,
    firmware_id: String,
    version_number: String,
    version_type: String,
    download_link: String,
    md5: String,
    content: String,
    importance: i64,
) -> Result<Value, String> {
    crate::tange::publish_version(
        &tange_cfg,
        &firmware_id,
        &version_number,
        &version_type,
        &download_link,
        &md5,
        &content,
        importance,
    )
    .await
}
