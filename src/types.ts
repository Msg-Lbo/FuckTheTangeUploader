/** SFTP 配置 */
export interface SftpConfig {
  host: string;
  port: number;
  username: string;
  password: string;
  remoteDir: string;
}

/** GenataTech 平台配置 */
export interface GenataConfig {
  username: string;
  password: string;
  baseUrl: string;
  productId: string;
}

/** Tange 平台配置 */
export interface TangeConfig {
  appId: string;
  accessKey: string;
  accessSecret: string;
  firmwareId: string;
}

/** 应用全局配置 */
export interface AppConfig {
  sftp: SftpConfig;
  genata: GenataConfig;
  tange: TangeConfig;
}

/** Genata 版本列表项 */
export interface VersionItem {
  id: number;
  version: string;
  filename: string;
  nextVersion: string | null;
  joinTime: string | null;
  user: string | null;
  fileSize: number | null;
  md5: string | null;
  description: string | null;
}

/** SFTP 远程文件条目 */
export interface SftpFileEntry {
  name: string;
  isDir: boolean;
  size: number;
}

/** Tange 登记版本参数 */
export interface TangePublishParams {
  firmwareId: string;
  versionNumber: string;
  versionType: string;
  downloadLink: string;
  md5: string;
  content: string;
  importance: number;
}

/** Tange 已有版本条目（查询验证接口返回的 data.list 元素，字段为 snake_case） */
export interface TangeVersionItem {
  firmware_id: string;
  version_number: string;
  version_type: string;
  download_link: string;
  file_size: number | null;
  md5: string;
  content: string;
  importance: number;
}

/** Tange 查询版本列表响应 */
export interface TangeVersionListResponse {
  code: number;
  data: {
    list: TangeVersionItem[];
    page: number;
    page_size: number;
    total: number;
    total_page: number;
  };
  msg: string;
  ok: number;
  status: string;
}
