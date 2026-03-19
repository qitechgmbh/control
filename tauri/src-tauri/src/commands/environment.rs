use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentInfo {
    qitech_os: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    qitech_os_git_timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    qitech_os_git_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    qitech_os_git_abbreviation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    qitech_os_git_url: Option<String>,
}

#[tauri::command]
pub async fn environment_get_info() -> Result<EnvironmentInfo, String> {
    Ok(EnvironmentInfo {
        qitech_os: std::env::var("QITECH_OS").unwrap_or_default() == "true",
        qitech_os_git_timestamp: std::env::var("QITECH_OS_GIT_TIMESTAMP").ok(),
        qitech_os_git_commit: std::env::var("QITECH_OS_GIT_COMMIT").ok(),
        qitech_os_git_abbreviation: std::env::var("QITECH_OS_GIT_ABBREVIATION").ok(),
        qitech_os_git_url: std::env::var("QITECH_OS_GIT_URL").ok(),
    })
}
