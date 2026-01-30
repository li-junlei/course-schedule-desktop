use serde::{Deserialize, Serialize};

/// 课程数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    /// 课程名称
    pub name: String,
    /// 教师名称
    pub teacher: String,
    /// 周次范围 [开始周, 结束周, 开始周2(可选), 结束周2(可选)]
    pub weeks: Vec<i32>,
    /// 单双周标记 (1=单周, 2=双周, 0=全周)
    pub week_type: i32,
    /// 星期 (1-7)
    pub day_of_week: i32,
    /// 节次范围 [开始节, 结束节]
    pub periods: Vec<i32>,
    /// 教室/地点
    pub location: String,
}

impl Course {
    /// 从原始数组解析课程数据
    /// 原始格式: [课程名, 教师, "1-16", "(单)", "一", "1-2", "地点"]
    pub fn from_raw_array(data: &[String]) -> Option<Self> {
        if data.len() < 7 {
            return None;
        }

        let name = data[0].clone();
        let teacher = data[1].clone();
        let location = data[6].clone();

        // 解析周次 "1-16" 或 "1-8,9-16"
        let weeks: Vec<i32> = data[2]
            .split(&[',', '-'][..])
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        // 解析单双周
        let week_type = match data[3].as_str() {
            "(单)" => 1,
            "(双)" => 2,
            _ => 0,
        };

        // 解析星期
        let day_of_week = match data[4].as_str() {
            "一" => 1,
            "二" => 2,
            "三" => 3,
            "四" => 4,
            "五" => 5,
            "六" => 6,
            "七" => 7,
            _ => 0,
        };

        // 解析节次 "1-2"
        let periods: Vec<i32> = data[5]
            .split('-')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        Some(Course {
            name,
            teacher,
            weeks,
            week_type,
            day_of_week,
            periods,
            location,
        })
    }

    /// 检查课程是否在指定周次显示
    pub fn is_visible_in_week(&self, week: i32) -> bool {
        // 直接检查周次列表中是否包含该周
        let in_range = self.weeks.contains(&week);

        // 检查单双周 (虽然 parser 已经展开了所有周次，但保留这个逻辑可以作为二次校验)
        let week_match = self.week_type == 0 || (self.week_type == 1 && week % 2 != 0) || (self.week_type == 2 && week % 2 == 0);

        in_range && week_match
    }
}

/// 用户凭证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

/// 登录初始化参数
#[derive(Debug, Clone)]
pub struct LoginInitParams {
    pub sessionid: String,
    pub deskey: String,
    pub randnumber: String,
    pub nowtime: String,
}

/// 登录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub cookie: Option<String>,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub background_image: Option<String>,
    pub first_day: Option<i64>, // 学期第一天的时间戳
    pub end_week: Option<i32>,  // 学期结束周次
    pub max_periods: Option<i32>, // 最大节次 (默认11)
    pub period_times: Option<Vec<PeriodTime>>, // 节次时间表
    pub edu_system_url: Option<String>, // 教务系统地址
    pub current_schedule_id: Option<String>, // 当前选中的课表ID
}

/// 时间段(包含开始和结束时间)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodTime {
    pub start: String, // 开始时间，格式: "8:00"
    pub end: String,   // 结束时间，格式: "8:45"
}

/// 课表元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleMetadata {
    pub id: String,              // 课表唯一ID
    pub name: String,            // 课表名称
    pub created_at: i64,         // 创建时间
    pub updated_at: i64,         // 更新时间
    pub course_count: usize,     // 课程数量
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            background_image: None,
            first_day: None,
            end_week: None,
            max_periods: Some(13), // Default to 13 to be safe
            period_times: None,
            edu_system_url: None,
            current_schedule_id: None,
        }
    }
}

/// 缓存的课表数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSchedule {
    pub id: String,              // 课表ID
    pub name: String,            // 课表名称
    pub courses: Vec<Course>,
    pub timestamp: i64,
    pub expire_time: i64,
}

impl CachedSchedule {
    /// 检查缓存是否过期（30天）
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now > self.expire_time
    }
}
