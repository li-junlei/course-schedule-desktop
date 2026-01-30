use crate::crypto::encode_login_params;
use crate::models::{Course, LoginInitParams, LoginResponse, UserCredentials};
use reqwest::Client;
use std::time::Duration;

/// 教务系统客户端
pub struct EduSystemClient {
    client: Client,
    base_url: String,
    cookie: Option<String>,
}

impl EduSystemClient {
    /// 创建新的客户端
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        EduSystemClient {
            client,
            base_url,
            cookie: None,
        }
    }

    /// 初始化登录，获取 sessionid、deskey、randnumber、nowtime
    pub async fn init_login(&mut self) -> Result<LoginInitParams, String> {
        let url = format!("{}{}", self.base_url, "/login/init");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP 错误: {}", response.status()));
        }

        // 解析 JSON 响应
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        let my_cookie = json["my_cookie"]
            .as_str()
            .ok_or("缺少 my_cookie 字段")?
            .to_string();
        let sessionid = json["sessionid"]
            .as_str()
            .ok_or("缺少 sessionid 字段")?
            .to_string();
        let deskey = json["deskey"].as_str().ok_or("缺少 deskey 字段")?.to_string();
        let randnumber = json["randnumber"]
            .as_str()
            .ok_or("缺少 randnumber 字段")?
            .to_string();
        let nowtime = json["nowtime"]
            .as_str()
            .ok_or("缺少 nowtime 字段")?
            .to_string();

        // 保存 cookie
        self.cookie = Some(my_cookie.clone());

        Ok(LoginInitParams {
            sessionid,
            deskey,
            randnumber,
            nowtime,
        })
    }

    /// 执行登录
    pub async fn login(
        &mut self,
        credentials: &UserCredentials,
    ) -> Result<LoginResponse, String> {
        // 1. 初始化登录，获取加密参数
        let init_params = self.init_login().await?;

        // 2. 编码登录参数
        let encoded_params = encode_login_params(
            &credentials.username,
            &credentials.password,
            &init_params.sessionid,
            &init_params.deskey,
            &init_params.randnumber,
            &init_params.nowtime,
        )?;

        // 3. 提交登录
        let url = format!("{}{}", self.base_url, "/login/submit");

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "cookie": self.cookie.as_ref().ok_or("缺少 Cookie")?,
                "data": encoded_params
            }))
            .send()
            .await
            .map_err(|e| format!("登录请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("登录失败: HTTP {}", response.status()));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析登录响应失败: {}", e))?;

        let status = json["status"].as_i64().unwrap_or(0);

        if status == 200 {
            Ok(LoginResponse {
                success: true,
                message: "登录成功".to_string(),
                cookie: self.cookie.clone(),
            })
        } else {
            Ok(LoginResponse {
                success: false,
                message: json["message"].as_str().unwrap_or("登录失败").to_string(),
                cookie: self.cookie.clone(),
            })
        }
    }

    /// 获取课表数据
    pub async fn get_schedule(&self) -> Result<Vec<Course>, String> {
        let cookie = self.cookie.as_ref().ok_or("未登录，缺少 Cookie")?;

        let url = format!("{}{}", self.base_url, "/schedule/get");

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "cookie": cookie }))
            .send()
            .await
            .map_err(|e| format!("获取课表失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("获取课表失败: HTTP {}", response.status()));
        }

        // 解析响应数据
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析课表数据失败: {}", e))?;

        // 响应应该是字符串 "kong" 或二维数组
        if let Some(data_str) = json.as_str() {
            if data_str == "kong" {
                return Err("Cookie 已过期，请重新登录".to_string());
            }
        }

        // 解析课表数组
        let courses_array = json
            .as_array()
            .ok_or("课表数据格式错误")?;

        let mut courses = Vec::new();
        for course_item in courses_array {
            if let Some(arr) = course_item.as_array() {
                let strings: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                if let Some(course) = Course::from_raw_array(&strings) {
                    courses.push(course);
                }
            }
        }

        Ok(courses)
    }

    /// 设置 Cookie（用于从存储中恢复）
    pub fn set_cookie(&mut self, cookie: &str) {
        self.cookie = Some(cookie.to_string());
    }
}
