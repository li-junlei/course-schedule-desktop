mod models;
mod crypto;
mod client;
mod storage;

use client::EduSystemClient;
use models::{AppConfig, CachedSchedule, Course, ScheduleMetadata, UserCredentials, TimeTable};
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
fn save_schedule_cache(
    courses: Vec<Course>, 
    name: String, 
    schedule_id: Option<String>, 
    first_day: Option<i64>,
    max_periods: Option<i32>,
    weeks_count: Option<i32>,
    time_table_id: Option<String>
) -> Result<String, String> {
    let storage = StorageManager::new()?;

    let now = Utc::now().timestamp();
    let expire_time = now + (30 * 24 * 60 * 60); // 30 天后过期

    // 生成或使用指定的 ID
    let id = if let Some(sid) = schedule_id {
        sid
    } else {
        StorageManager::generate_schedule_id()
    };

    // 如果是新课表，需要计算 sort_index
    let mut sort_index = None;
    if let Ok(existing) = storage.load_schedule(&id) {
        sort_index = existing.sort_index;
    } 
    if sort_index.is_none() {
        let current_list = storage.list_schedules()?;
        let max_index = current_list.iter().filter_map(|s| s.sort_index).max().unwrap_or(-1);
        sort_index = Some(max_index + 1);
    }

    let cached = CachedSchedule {
        id: id.clone(),
        name,
        courses,
        timestamp: now,
        expire_time,
        first_day,
        max_periods,
        weeks_count,
        time_table_id,
        sort_index,
    };

    storage.save_schedule(&cached)?;

    // 更新当前选中的课表 ID
    let mut config = storage.load_config().unwrap_or_default();
    config.current_schedule_id = Some(id.clone());

    // 如果提供了 first_day，也更新全局配置 (兼容性)
    if let Some(fd) = first_day {
        config.first_day = Some(fd);
    }

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
    config.current_schedule_id = Some(schedule_id.clone());

    // 同步课表的 first_day 到全局配置
    let schedule = storage.load_schedule(&schedule_id)?;
    config.first_day = schedule.first_day;

    storage.save_config(&config)?;
    Ok(())
}

/// 重新排序课表
#[tauri::command]
fn reorder_schedules(sorted_ids: Vec<String>) -> Result<(), String> {
    let storage = StorageManager::new()?;

    for (index, id) in sorted_ids.iter().enumerate() {
        if let Ok(mut schedule) = storage.load_schedule(id) {
            schedule.sort_index = Some(index as i32);
            storage.save_schedule(&schedule)?;
        }
    }

    Ok(())
}

/// 更新课表信息
#[tauri::command]
fn update_schedule_info(
    schedule_id: String, 
    first_day: Option<i64>,
    max_periods: Option<i32>,
    weeks_count: Option<i32>,
    time_table_id: Option<String>
) -> Result<(), String> {
    println!("更新课表信息 - schedule_id: {}", schedule_id);

    let storage = StorageManager::new()?;

    // 加载课表
    let mut cached = storage.load_schedule(&schedule_id)?;
    
    // 更新字段
    cached.first_day = first_day;
    cached.max_periods = max_periods;
    cached.weeks_count = weeks_count;
    cached.time_table_id = time_table_id;

    // 保存
    storage.save_schedule(&cached)?;
    
    // 如果这是当前选中的课表，也更新全局配置 (兼容性)
    let config = storage.load_config()?;
    if let Some(ref current_id) = config.current_schedule_id {
        if current_id == &schedule_id {
            let mut new_config = config;
            new_config.first_day = first_day;
            if let Some(mp) = max_periods { new_config.max_periods = Some(mp); }
            storage.save_config(&new_config)?;
        }
    }

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

/// 获取应用配置 (含自动迁移逻辑)
#[tauri::command]
fn get_app_config() -> Result<AppConfig, String> {
    let storage = StorageManager::new()?;
    let mut config = storage.load_config()?;

    // 自动迁移：如果 time_tables 为空但 period_times 存在，创建默认时间表
    let has_time_tables = config.time_tables.as_ref().map_or(false, |v| !v.is_empty());
    if !has_time_tables {
        if let Some(ref periods) = config.period_times {
            let default_table = TimeTable {
                id: "default".to_string(),
                name: "默认时间表".to_string(),
                periods: periods.clone(),
            };
            config.time_tables = Some(vec![default_table]);
            storage.save_config(&config)?;
        }
    }

    Ok(config)
}

/// 保存应用配置
#[tauri::command]
fn save_app_config(config: AppConfig) -> Result<(), String> {
    let storage = StorageManager::new()?;
    storage.save_config(&config)?;
    Ok(())
}

/// 保存时间表
#[tauri::command]
fn save_time_table(time_table: TimeTable) -> Result<(), String> {
    let storage = StorageManager::new()?;
    let mut config = storage.load_config().unwrap_or_default();

    let mut tables = config.time_tables.unwrap_or_default();
    
    // 如果ID已存在则更新，否则添加
    if let Some(index) = tables.iter().position(|t| t.id == time_table.id) {
        tables[index] = time_table;
    } else {
        tables.push(time_table);
    }
    
    config.time_tables = Some(tables);
    storage.save_config(&config)?;
    Ok(())
}

/// 删除时间表
#[tauri::command]
fn delete_time_table(id: String) -> Result<(), String> {
    let storage = StorageManager::new()?;
    let mut config = storage.load_config().unwrap_or_default();

    if let Some(tables) = config.time_tables {
        let new_tables: Vec<TimeTable> = tables.into_iter().filter(|t| t.id != id).collect();
        config.time_tables = Some(new_tables);
        storage.save_config(&config)?;
    }
    Ok(())
}

/// 获取时间表列表
#[tauri::command]
fn list_time_tables() -> Result<Vec<TimeTable>, String> {
    let config = get_app_config()?;
    Ok(config.time_tables.unwrap_or_default())
}

/// 将指定课表的设置应用到所有课表
#[tauri::command]
fn apply_settings_to_all(source_schedule_id: String) -> Result<(), String> {
    let storage = StorageManager::new()?;
    
    // 1. 获取源课表设置
    let source = storage.load_schedule(&source_schedule_id)?;
    // 注意：不应用 first_day，因为不同学期或不同用户的课表起始日可能不同
    let max_periods = source.max_periods;
    let weeks_count = source.weeks_count;
    let time_table_id = source.time_table_id;
    
    // 2. 获取所有课表ID
    let metadata_list = storage.list_schedules()?;
    
    // 3. 遍历更新
    for meta in metadata_list {
        if meta.id == source_schedule_id {
            continue;
        }
        
        let mut schedule = storage.load_schedule(&meta.id)?;
        // schedule.first_day = first_day; // SKIP
        schedule.max_periods = max_periods;
        schedule.weeks_count = weeks_count;
        schedule.time_table_id = time_table_id.clone();
        
        storage.save_schedule(&schedule)?;
    }
    
    // 4. 更新全局配置 (如果存在相关项)
    let mut config = storage.load_config()?;
    // config.first_day = first_day; // SKIP
    config.max_periods = max_periods;
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
async fn import_from_browser(app: tauri::AppHandle, html: String, name: Option<String>) -> Result<String, String> {
    println!("=== 浏览器导入被调用 ===");
    println!("HTML 长度: {} 字符", html.len());
    println!("名称参数: {:?}", name);

    // 解析 HTML
    let courses = parser::parse_course_html(&html)?;
    println!("解析到 {} 门课程", courses.len());

    // 生成课表ID
    let schedule_id = StorageManager::generate_schedule_id();

    // 使用提供的名称或默认名称
    let schedule_name = name.unwrap_or_else(|| {
        format!("从浏览器导入 {}", chrono::Utc::now().format("%Y-%m-%d"))
    });

    println!("课表名称: {}", schedule_name);

    // 保存课表
    let storage = StorageManager::new()?;
    let now = Utc::now().timestamp();
    let expire_time = now + (30 * 24 * 60 * 60); // 30 天后过期

    // 计算 sort_index (放在最后)
    let current_list = storage.list_schedules()?;
    let max_index = current_list.iter().filter_map(|s| s.sort_index).max().unwrap_or(-1);
    let sort_index = max_index + 1;

    let cached = CachedSchedule {
        id: schedule_id.clone(),
        name: schedule_name,
        courses,
        timestamp: now,
        expire_time,
        first_day: None, 
        max_periods: None,
        weeks_count: None,
        time_table_id: None,
        sort_index: Some(sort_index),
    };

    storage.save_schedule(&cached)?;
    println!("课表已保存到文件");

    // 更新当前选中的课表 ID
    let mut config = storage.load_config().unwrap_or_default();
    config.current_schedule_id = Some(schedule_id.clone());
    storage.save_config(&config)?;
    println!("全局配置已更新");

    // 通过事件通知前端，导入成功并返回课表ID和课程数量
    app.emit("schedule-imported", serde_json::json!({
        "schedule_id": schedule_id,
        "course_count": cached.courses.len(),
        "schedule_name": cached.name
    }))
        .map_err(|e| format!("发送事件失败: {}", e))?;

    println!("=== 浏览器导入完成 ===");
    Ok(schedule_id)
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
            reorder_schedules,
            update_schedule_info,
            save_time_table,
            delete_time_table,
            list_time_tables,
            apply_settings_to_all,
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
            get_window_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 检查窗口是否存在
#[tauri::command]
fn get_window_state(app: tauri::AppHandle, window_label: String) -> Result<String, String> {
    if let Some(_window) = app.get_webview_window(&window_label) {
        Ok("exists".to_string())
    } else {
        Err("Window not found".to_string())
    }
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

                        btn.innerHTML = '⏳ 正在复制...';
                        btn.style.backgroundColor = '#E6A23C';

                        // 尝试使用 Clipboard API
                        if (navigator.clipboard && navigator.clipboard.writeText) {
                            await navigator.clipboard.writeText(html);
                            console.log('HTML 已复制到剪贴板');
                            btn.innerHTML = '✅ 已复制！请关闭此窗口';
                            btn.style.backgroundColor = '#67C23A';

                            // 禁用按钮，防止重复点击
                            btn.disabled = true;
                            btn.style.cursor = 'default';
                            btn.onclick = null;
                        } else {
                            // 降级方案：使用传统方法
                            const textArea = document.createElement('textarea');
                            textArea.value = html;
                            textArea.style.position = 'fixed';
                            textArea.style.left = '-999999px';
                            document.body.appendChild(textArea);
                            textArea.select();

                            try {
                                document.execCommand('copy');
                                console.log('HTML 已复制到剪贴板（传统方法）');
                                btn.innerHTML = '✅ 已复制！请关闭此窗口';
                                btn.style.backgroundColor = '#67C23A';
                                document.body.removeChild(textArea);

                                // 禁用按钮，防止重复点击
                                btn.disabled = true;
                                btn.style.cursor = 'default';
                                btn.onclick = null;
                            } catch (err) {
                                console.error('复制失败:', err);
                                btn.innerHTML = '❌ 复制失败';
                                btn.style.backgroundColor = '#F56C6C';
                                document.body.removeChild(textArea);

                                setTimeout(() => {
                                    btn.innerHTML = '📥 导入当前课表';
                                    btn.style.backgroundColor = '#409EFF';
                                }, 3000);
                            }
                        }
                    } catch (err) {
                        console.error('导入失败:', err);
                        btn.innerHTML = '❌ 复制失败';
                        btn.style.backgroundColor = '#F56C6C';

                        setTimeout(() => {
                            btn.innerHTML = '📥 导入当前课表';
                            btn.style.backgroundColor = '#409EFF';
                        }, 3000);
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
