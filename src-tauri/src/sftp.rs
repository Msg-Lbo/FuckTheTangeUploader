use std::io::Read;
use std::sync::Arc;

use russh::client;
use russh::keys;
use russh_sftp::client::SftpSession;
use russh_sftp::protocol::OpenFlags;
use serde::Serialize;
use tokio::io::AsyncWriteExt;

use crate::config::SftpConfig;

/** russh 客户端 Handler（首次连接接受任意服务器密钥） */
struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

/** 远程文件条目 */
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpFileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

/**
 * 建立 SFTP 连接并认证
 * @param {SftpConfig} cfg - SFTP 配置
 * 返回 (SSH 会话, SFTP 会话)，SSH 会话需在 SFTP 操作完成前保持存活
 */
async fn connect(cfg: &SftpConfig) -> Result<(client::Handle<ClientHandler>, SftpSession), String> {
    let config = Arc::new(client::Config::default());
    let mut session = client::connect(config, (cfg.host.as_str(), cfg.port), ClientHandler)
        .await
        .map_err(|e| format!("连接 SSH 服务器失败: {e}"))?;

    let auth = session
        .authenticate_password(&cfg.username, &cfg.password)
        .await
        .map_err(|e| format!("SSH 认证失败: {e}"))?;
    if !auth.success() {
        return Err("SSH 认证未通过，请检查用户名/密码".to_string());
    }

    let channel = session
        .channel_open_session()
        .await
        .map_err(|e| format!("打开 SSH 通道失败: {e}"))?;
    channel
        .request_subsystem(true, "sftp")
        .await
        .map_err(|e| format!("请求 SFTP 子系统失败: {e}"))?;
    let sftp = SftpSession::new(channel.into_stream())
        .await
        .map_err(|e| format!("初始化 SFTP 会话失败: {e}"))?;

    Ok((session, sftp))
}

/**
 * 计算本地文件的 MD5（小写 hex）
 * @param {String} path - 本地文件路径
 * 返回 MD5 字符串
 */
pub fn md5_of_file(path: &str) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|e| format!("打开文件失败: {e}"))?;
    let mut ctx = md5::Context::new();
    let mut buf = vec![0u8; 64 * 1024]; // 64KB 分块读取，避免大文件撑爆内存
    loop {
        let n = file.read(&mut buf).map_err(|e| format!("读取文件失败: {e}"))?;
        if n == 0 {
            break;
        }
        ctx.consume(&buf[..n]);
    }
    Ok(format!("{:x}", ctx.compute()))
}

/**
 * 通过 SFTP 上传本地文件到远程目录
 * @param {String} local_path - 本地文件路径
 * @param {String} remote_filename - 远程文件名
 * @param {SftpConfig} cfg - SFTP 配置
 */
pub async fn upload(local_path: &str, remote_filename: &str, cfg: &SftpConfig) -> Result<(), String> {
    let (_session, sftp) = connect(cfg).await?;

    // 拼接远程路径（remote_dir 可为空，表示根目录）
    let remote_dir = if cfg.remote_dir.is_empty() {
        String::new()
    } else {
        format!("{}/", cfg.remote_dir.trim_end_matches('/'))
    };
    let remote_path = format!("{remote_dir}{remote_filename}");

    let mut remote = sftp
        .open_with_flags(
            remote_path.as_str(),
            OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE,
        )
        .await
        .map_err(|e| format!("创建远程文件失败: {e}"))?;

    let data = tokio::fs::read(local_path)
        .await
        .map_err(|e| format!("读取本地文件失败: {e}"))?;

    remote
        .write_all(&data)
        .await
        .map_err(|e| format!("写入远程文件失败: {e}"))?;
    remote.flush().await.map_err(|e| format!("刷新远程文件失败: {e}"))?;
    remote.shutdown().await.map_err(|e| format!("关闭远程文件失败: {e}"))?;

    Ok(())
}

/**
 * 列出远程目录下的文件
 * @param {SftpConfig} cfg - SFTP 配置
 * @param {String} remote_dir - 远程目录（空表示根目录）
 * 返回文件条目数组
 */
pub async fn list_files(cfg: &SftpConfig, remote_dir: &str) -> Result<Vec<SftpFileEntry>, String> {
    let (_session, sftp) = connect(cfg).await?;
    let dir = if remote_dir.is_empty() { "." } else { remote_dir };

    let entries = sftp
        .read_dir(dir)
        .await
        .map_err(|e| format!("读取目录失败: {e}"))?;

    let mut files = Vec::new();
    for entry in entries {
        let meta = entry.metadata();
        files.push(SftpFileEntry {
            name: entry.file_name(),
            is_dir: meta.is_dir(),
            size: meta.len(),
        });
    }
    Ok(files)
}
