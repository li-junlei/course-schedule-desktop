mod models;
mod crypto;
mod client;
mod storage;

use client::EduSystemClient;
use models::{AppConfig, CachedSchedule, Course, ScheduleMetadata, UserCredentials};
use storage::StorageManager;
use chrono::{Utc, Duration, Datelike};
use std::fs;
use tauri::{Manager, Emitter};

// ============== Tauri Commands ==============

/// 登录并获取课表
#[tauri::command]
async fn login_and_get_schedule(
    username: String,
    password: String,
    base_url: String,
) -> Result<Vec<Course>, String> {
    let credentials = UserCredentials { username, password };
    let mut client = EduSystemClient::new(base_url);

    // 执行登录
    let login_response = client.login(&credentials).await?;
    if !login_response.success {
        return Err(format!("登录失败: {}", login_response.message));
    }

    // 获取课表
    let courses = client.get_schedule().await?;

    // 保存 Cookie
    if let Some(cookie) = login_response.cookie {
        let storage = StorageManager::new()?;
        storage.save_cookie(&cookie)?;
    }

    Ok(courses)
}

/// 刷新课表数据（使用已保存的 Cookie）
#[tauri::command]
async fn refresh_schedule(base_url: String) -> Result<Vec<Course>, String> {
    let storage = StorageManager::new()?;

    // 加载 Cookie
    let cookie = storage.load_cookie()?;

    // 创建客户端并设置 Cookie
    let mut client = EduSystemClient::new(base_url);
    client.set_cookie(&cookie);

    // 获取课表
    let courses = client.get_schedule().await?;

    Ok(courses)
}

/// 加载缓存的课表数据
#[tauri::command]
fn load_cached_schedule(schedule_id: Option<String>) -> Result<Vec<Course>, String> {
    let storage = StorageManager::new()?;

    // 如果没有指定 ID,尝试从配置中获取
    let id = if let Some(sid) = schedule_id {
        sid
    } else {
        let config = storage.load_config()?;
        if let Some(current_id) = config.current_schedule_id {
            current_id
        } else {
            return Err("没有选中的课表".to_string());
        }
    };

    let cached = storage.load_schedule(&id)?;

    if cached.is_expired() {
        return Err("课表数据已过期，请刷新".to_string());
    }

    Ok(cached.courses)
}

/// 保存课表数据到缓存
#[tauri::command]
fn save_schedule_cache(courses: Vec<Course>, name: String, schedule_id: Option<String>) -> Result<String, String> {
    let storage = StorageManager::new()?;

    let now = Utc::now().timestamp();
    let expire_time = now + (30 * 24 * 60 * 60); // 30 天后过期

    // 生成或使用指定的 ID
    let id = if let Some(sid) = schedule_id {
        sid
    } else {
        StorageManager::generate_schedule_id()
    };

    let cached = CachedSchedule {
        id: id.clone(),
        name,
        courses,
        timestamp: now,
        expire_time,
    };

    storage.save_schedule(&cached)?;

    // 更新当前选中的课表 ID
    let mut config = storage.load_config().unwrap_or_default();
    config.current_schedule_id = Some(id.clone());
    storage.save_config(&config)?;

    Ok(id)
}

/// 获取所有课表列表
#[tauri::command]
fn list_schedules() -> Result<Vec<ScheduleMetadata>, String> {
    let storage = StorageManager::new()?;
    storage.list_schedules()
}

/// 删除指定课表
#[tauri::command]
fn delete_schedule(schedule_id: String) -> Result<(), String> {
    let storage = StorageManager::new()?;

    // 获取当前配置
    let mut config = storage.load_config().unwrap_or_default();

    // 如果删除的是当前选中的课表,清空选中状态
    if let Some(current_id) = &config.current_schedule_id {
        if current_id == &schedule_id {
            config.current_schedule_id = None;
            storage.save_config(&config)?;
        }
    }

    storage.delete_schedule(&schedule_id)
}

/// 切换当前课表
#[tauri::command]
fn switch_schedule(schedule_id: String) -> Result<(), String> {
    let storage = StorageManager::new()?;
    let mut config = storage.load_config().unwrap_or_default();
    config.current_schedule_id = Some(schedule_id);
    storage.save_config(&config)?;
    Ok(())
}

/// 获取当前周次
#[tauri::command]
fn get_current_week(first_day: Option<i64>) -> i32 {
    if let Some(first_day) = first_day {
        let now = Utc::now().timestamp();
        let weeks = ((now - first_day) / (7 * 24 * 60 * 60)) + 1;
        weeks.max(1) as i32
    } else {
        1
    }
}

/// 计算指定周次和星期的日期
/// 返回格式: "M/D" (如 "1/15")
#[tauri::command]
fn calculate_date(first_day: i64, target_week: i32, target_day: i32) -> String {
    let first_date = chrono::DateTime::<Utc>::from_timestamp(first_day, 0)
        .unwrap()
        .date_naive();

    // 计算目标日期
    let target_date = first_date
        + Duration::weeks((target_week - 1) as i64)
        + Duration::days((target_day - 1) as i64);

    format!("{}/{}", target_date.month(), target_date.day())
}

/// 保存背景图
#[tauri::command]
async fn save_background_image(source_path: String) -> Result<String, String> {
    let storage = StorageManager::new()?;
    let bg_dir = storage.background_dir();

    // 读取源文件
    let file_name = format!("bg_{}.jpg", Utc::now().timestamp());
    let dest_path = bg_dir.join(&file_name);

    fs::copy(&source_path, &dest_path)
        .map_err(|e| format!("复制背景图失败: {}", e))?;

    // 保存到配置
    let mut config = storage.load_config().unwrap_or_default();
    config.background_image = Some(file_name.clone());
    storage.save_config(&config)?;

    Ok(file_name)
}

/// 删除背景图
#[tauri::command]
fn delete_background_image() -> Result<(), String> {
    let storage = StorageManager::new()?;
    let config = storage.load_config()?;

    if let Some(filename) = config.background_image {
        storage.delete_background(&filename)?;
    }

    // 更新配置
    let mut config = storage.load_config().unwrap_or_default();
    config.background_image = None;
    storage.save_config(&config)?;

    Ok(())
}

/// 获取应用配置
#[tauri::command]
fn get_app_config() -> Result<AppConfig, String> {
    let storage = StorageManager::new()?;
    let config = storage.load_config()?;
    Ok(config)
}

/// 保存应用配置
#[tauri::command]
fn save_app_config(config: AppConfig) -> Result<(), String> {
    let storage = StorageManager::new()?;
    storage.save_config(&config)?;
    Ok(())
}

/// 清空所有数据
#[tauri::command]
fn clear_all_data() -> Result<(), String> {
    let storage = StorageManager::new()?;
    storage.clear_all()?;
    Ok(())
}

// ============== Main Entry Point ==============

mod parser; 

// ... existing code ...

/// 解析 HTML 课表
#[tauri::command]
fn parse_html_schedule(html: String) -> Result<Vec<Course>, String> {
    parser::parse_course_html(&html)
}

/// 从浏览器导入课表
#[tauri::command]
async fn import_from_browser(app: tauri::AppHandle, html: String) -> Result<(), String> {
    // 解析 HTML
    let courses = parser::parse_course_html(&html)?;

    // 通过事件发送课程数据到前端
    app.emit("schedule-imported", courses)
        .map_err(|e| format!("发送事件失败: {}", e))?;

    Ok(())
}

// ============== Main Entry Point ==============

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // 登录相关
            login_and_get_schedule,
            refresh_schedule,
            load_cached_schedule,
            save_schedule_cache,
            // 课表管理
            list_schedules,
            delete_schedule,
            switch_schedule,
            // 数据相关
            get_current_week,
            calculate_date,
            parse_html_schedule,
            import_from_browser,
            // 配置相关
            save_background_image,
            delete_background_image,
            get_app_config,
            save_app_config,
            clear_all_data,
            open_login_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 打开登录窗口
#[tauri::command]
async fn open_login_window(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    let win_label = "login_window";
    
    // 如果窗口已存在，聚焦
    if let Some(win) = app.get_webview_window(win_label) {
        win.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let script = r#"
        (function() {
            // Override window.open to force same-window navigation
            // Many edu systems use window.open for sub-modules
            window.open = function(url) {
                if (url) {
                    // Handle relative URLs
                    window.location.href = url;
                }
                return window;
            };

            // Fix links to open in same window (Baidu defaults to _blank)
            function fixLinks() {
                try {
                    const links = document.querySelectorAll('a[target="_blank"]');
                    links.forEach(link => {
                        try { link.target = '_self'; } catch(e) {}
                    });
                     // Also fix common frame layouts if accessible
                    try {
                        const frames = document.querySelectorAll('iframe');
                        frames.forEach(frame => {
                            try {
                                const fDoc = frame.contentDocument;
                                if (fDoc) {
                                     const fLinks = fDoc.querySelectorAll('a[target="_blank"]');
                                     fLinks.forEach(l => l.target = '_self');
                                }
                            } catch(e) {}
                        });
                    } catch(e) {}
                } catch(e) {}
            }

            // Run frequently to catch dynamic content
            setInterval(fixLinks, 1000);
            window.addEventListener('DOMContentLoaded', fixLinks);
            window.addEventListener('load', fixLinks);
            document.addEventListener('click', function(e) {
                // aggressively fix target if clicking an anchor
                let target = e.target;
                while (target && target.tagName !== 'A') {
                    target = target.parentElement;
                }
                if (target && target.tagName === 'A') {
                    target.target = '_self';
                }
            }, true);

            // Function to add the button
            function addImportBtn() {
                if (document.getElementById('tauri-import-btn')) return;

                const btn = document.createElement('button');
                btn.id = 'tauri-import-btn';
                btn.innerHTML = '📥 导入当前课表';
                btn.style.cssText = `
                    position: fixed;
                    bottom: 30px;
                    right: 30px;
                    z-index: 2147483647;
                    padding: 12px 24px;
                    background-color: #409EFF;
                    color: white;
                    border: none;
                    border-radius: 8px;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.2);
                    font-size: 16px;
                    font-weight: bold;
                    cursor: pointer;
                    transition: all 0.3s;
                `;

                btn.onmouseover = () => btn.style.transform = 'scale(1.05)';
                btn.onmouseout = () => btn.style.transform = 'scale(1)';

                btn.onclick = async function() {
                    try {
                        const html = document.documentElement.outerHTML;

                        // 通过 Tauri 事件发送 HTML 到前端
                        if (window.__TAURI__?.core) {
                            btn.innerHTML = '⏳ 正在导入...';
                            btn.style.backgroundColor = '#E6A23C';

                            // 调用 Tauri 命令发送 HTML
                            await window.__TAURI__.core.invoke('import_from_browser', { html });

                            btn.innerHTML = '✅ 导入成功！';
                            btn.style.backgroundColor = '#67C23A';

                            setTimeout(() => {
                                btn.innerHTML = '📥 导入当前课表';
                                btn.style.backgroundColor = '#409EFF';
                            }, 2000);
                        } else {
                            // 如果 Tauri API 不可用,使用剪贴板作为后备
                            await navigator.clipboard.writeText(html);
                            alert('课表数据已复制到剪贴板！\n请返回软件主界面进行解析。');
                        }
                    } catch (err) {
                        console.error('Import failed', err);
                        btn.innerHTML = '❌ 导入失败';
                        btn.style.backgroundColor = '#F56C6C';
                        setTimeout(() => {
                            btn.innerHTML = '📥 导入当前课表';
                            btn.style.backgroundColor = '#409EFF';
                        }, 2000);
                    }
                };

                document.body.appendChild(btn);
            }

            // Observe DOM changes to ensure button persists
            const observer = new MutationObserver((mutations) => {
                if (!document.body) return;
                addImportBtn();
            });

            if (document.body) {
                addImportBtn();
                observer.observe(document.body, { childList: true, subtree: true });
            } else {
                window.addEventListener('DOMContentLoaded', () => {
                    addImportBtn();
                    observer.observe(document.body, { childList: true, subtree: true });
                });
            }
        })();
    "#;

    WebviewWindowBuilder::new(&app, win_label, WebviewUrl::External(url::Url::parse(&url).map_err(|e| e.to_string())?))
        .title("教务系统 - 登录并导入")
        .inner_size(1024.0, 768.0)
        .initialization_script(script)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}
