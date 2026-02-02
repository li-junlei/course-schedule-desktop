use scraper::{Html, Selector, ElementRef};
use regex::Regex;
use std::collections::HashMap;
use crate::models::Course;

/// 解析HTML课程表（中央财经大学）
pub fn parse_course_html(html_content: &str) -> Result<Vec<Course>, String> {
    let document = Html::parse_document(html_content);

    // 查找包含 timetable 类的表格 (Python: re.compile(r'timetable'))
    // scraper 不支持正则选择器，但通常类名就是 timetable 或包含它。
    // 这里我们尝试找所有table，然后检查class
    let mut course_table = None;
    let table_sel = Selector::parse("table").unwrap();

    for table in document.select(&table_sel) {
        if let Some(class_attr) = table.value().attr("class") {
            if class_attr.contains("timetable") {
                course_table = Some(table);
                break;
            }
        }
    }

    let table = course_table.ok_or("未找到课程表表格 (table with class 'timetable')")?;

    let tr_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let div_con_selector = Selector::parse("div.timetable_con").unwrap();

    let rows: Vec<ElementRef> = table.select(&tr_selector).collect();
    if rows.len() < 3 {
        return Err("表格行数不足".to_string());
    }

    let mut courses = Vec::new();
    let mut col_rowspan: HashMap<usize, usize> = HashMap::new();

    // 预定义的星期列表
    let day_list = vec!["周一", "周二", "周三", "周四", "周五", "周六", "周日"];



    let mut current_period_str = String::new();

    // 从第3行开始 (rows[2])
    for row in rows.into_iter().skip(2) {
        let cells: Vec<ElementRef> = row.select(&td_selector).collect();
        if cells.is_empty() {
            continue;
        }

        let first_cell_text = cells[0].text().collect::<Vec<_>>().join("").trim().to_string();
        
        // 逻辑分支：检查第一列是否是时间段（上午/下午/晚上）或直接是节次
        let is_time_slot = ["上午", "下午", "晚上", "中午"].contains(&first_cell_text.as_str());
        
        let start_cell_idx;
        let mut day_idx = 0;

        if is_time_slot {
             // 这种情况下节次在第2列（索引1）
             if cells.len() >= 2 {
                let period_text = cells[1].text().collect::<Vec<_>>().join("").trim().to_string();
                if period_text.chars().all(char::is_numeric) {
                    current_period_str = period_text;
                }
             }
             start_cell_idx = 2; // 从第3列开始是星期
        } else if first_cell_text.chars().all(char::is_numeric) {
            // 第1列就是节次号
            current_period_str = first_cell_text;
            start_cell_idx = 1; // 从第2列开始是星期
        } else {
            // 其他情况，可能这行不包含节次信息，跳过或处理
            continue; 
        }

        // 遍历每一天的单元格
        // Python: for cell_idx, cell in enumerate(cells[start_cell_idx:], start=start_cell_idx):
        for cell in cells.into_iter().skip(start_cell_idx) {
            // 跳过被 rowspan 占据的列
            while let Some(&count) = col_rowspan.get(&day_idx) {
                if count > 0 {
                    col_rowspan.insert(day_idx, count - 1);
                    if count - 1 == 0 {
                        col_rowspan.remove(&day_idx);
                    }
                    day_idx += 1;
                } else {
                    col_rowspan.remove(&day_idx);
                }
            }

            if day_idx < day_list.len() {
                let day_str = day_list[day_idx];
                
                let mut rs = 1;
                // 检查并记录当前单元格的 rowspan
                if let Some(rowspan_attr) = cell.value().attr("rowspan") {
                    if let Ok(val) = rowspan_attr.parse::<usize>() {
                        rs = val;
                        if rs > 1 {
                             col_rowspan.insert(day_idx, rs - 1);
                        }
                    }
                }

                // 解析课程内容
                if let Some(course_div) = cell.select(&div_con_selector).next() {
                    if let Some(course) = parse_course_cell(&course_div, day_str, &current_period_str, rs) {
                        courses.push(course);
                    }
                }
                
                day_idx += 1;
            }
        }
        
    }

    Ok(courses)
}

fn parse_course_cell(
    div: &ElementRef, 
    day_str: &str, 
    period_str: &str,
    rowspan: usize
) -> Option<Course> {
    let span_title_selector = Selector::parse("span.title, u.title").unwrap();
    let p_selector = Selector::parse("p").unwrap();
    
    // 提取课程名称
    let title_elem = div.select(&span_title_selector).next()?;
    let mut title_text = title_elem.text().collect::<Vec<_>>().join("").trim().to_string();
    title_text = title_text.replace("【调】", "").trim().to_string();

    // 课程类型映射
    let type_map = HashMap::from([
        ("★", "讲课"),
        ("○", "实验"),
        ("◆", "讨论"),
        ("◇", "上机"),
        ("●", "实践"),
    ]);

    // 清理标题中的符号
    for symbol in type_map.keys() {
        if title_text.ends_with(symbol) {
             title_text = title_text.replace(symbol, "").trim().to_string();
        }
    }
    let course_name = title_text;

    // 提取详细信息
    let mut weeks_vec: Vec<i32> = Vec::new();
    let mut classroom = String::from("未指定");
    let mut teacher = String::from("未指定");
    
    // 解析p标签
    for p in div.select(&p_selector) {
        let text = p.text().collect::<Vec<_>>().join("").trim().to_string();
        let span_sel = Selector::parse("span").unwrap();
        let title_attr = if let Some(span) = p.select(&span_sel).next() {
            span.value().attr("title").unwrap_or("").to_string()
        } else {
             p.value().attr("title").unwrap_or("").to_string()
        };

        if text.contains("节") && text.contains("周") {
            // 提取周次: "(1-2节)1-7周,11-16周"
            let re = Regex::new(r"(\d+(?:-\d+)?)\s*周").unwrap();
            for cap in re.captures_iter(&text) {
                if let Some(m) = cap.get(1) {
                    let range_str = m.as_str();
                    if range_str.contains('-') {
                        let parts: Vec<&str> = range_str.split('-').collect();
                        if parts.len() == 2 {
                            if let (Ok(start), Ok(end)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                                if start <= end {
                                    for w in start..=end {
                                        weeks_vec.push(w);
                                    }
                                }
                            }
                        }
                    } else if let Ok(w) = range_str.parse::<i32>() {
                        weeks_vec.push(w);
                    }
                }
            }
        } else if title_attr.contains("上课地点") || text.contains("主教") || text.contains("校区") || text.contains("楼") {
            classroom = text.replace("学院南路校区", "").replace("沙河校区", "").trim().to_string();
        } else if title_attr.contains("教师") {
            teacher = text.trim().to_string();
        }
    }
    
    // 处理周次向量并排序去重
    weeks_vec.sort();
    weeks_vec.dedup();

    // 解析星期 str -> i32
    let day_of_week = match day_str {
        "周一" => 1,
        "周二" => 2,
        "周三" => 3,
        "周四" => 4,
        "周五" => 5,
        "周六" => 6,
        "周日" => 7,
        _ => 1,
    };

    // 解析节次 (基于 rowspan)
    // 逻辑：Start Period + Rowspan
    let start_period = if let Ok(p) = period_str.parse::<i32>() {
        p
    } else if let Some(first_part) = period_str.split('-').next() {
        first_part.parse::<i32>().unwrap_or(0)
    } else {
        0
    };

    let periods: Vec<i32> = if start_period > 0 {
        (start_period .. start_period + rowspan as i32).collect()
    } else {
        vec![]
    };

    let week_type = 0;

    Some(Course {
        name: course_name,
        teacher,
        weeks: weeks_vec,
        week_type,
        day_of_week,
        periods,
        location: classroom,
    })
}

/// 解析浙江大学课程表 HTML
/// 严格按照 C:\project\zjucourses\extract_schedule.py 的逻辑实现
pub fn parse_zju_html(html_content: &str) -> Result<Vec<Course>, String> {
    let document = Html::parse_document(html_content);

    // 查找课表表格 ID="kbgrid_table"
    let table_sel = Selector::parse("table#kbgrid_table").unwrap();
    let table = document
        .select(&table_sel)
        .next()
        .ok_or("未找到浙江大学课表表格 (table#kbgrid_table)")?;

    let td_selector = Selector::parse("td[id]").unwrap();
    let a_selector = Selector::parse("a[onclick*='showCourseInfo2']").unwrap();
    let font_selector = Selector::parse("font[color='blue']").unwrap();

    // 存储原始课程记录（包含单双周）
    let mut raw_courses: Vec<RawZJUCourse> = Vec::new();
    let mut processed_cells: HashMap<String, bool> = HashMap::new();

    // 遍历所有包含课程的单元格
    for td in table.select(&td_selector) {
        let cell_id = td.value().attr("id").unwrap_or("");

        // 跳过已处理的单元格
        if processed_cells.contains_key(cell_id) {
            continue;
        }

        // 解析单元格 ID：格式为 {星期}-{单双周}-{节次}
        // 例如：2-1-1 表示 周二-双周-第1节
        let parts: Vec<&str> = cell_id.split('-').collect();
        if parts.len() != 3 {
            continue;
        }

        let weekday: i32 = parts[0].parse().unwrap_or(0);
        let parity_code: i32 = parts[1].parse().unwrap_or(0);
        let period: i32 = parts[2].parse().unwrap_or(0);

        if weekday == 0 || period == 0 {
            continue;
        }

        // 0=单周, 1=双周
        let parity_str = if parity_code == 0 { "单" } else { "双" };

        // 查找课程链接
        if let Some(course_link) = td.select(&a_selector).next() {
            if let Some(font_tag) = course_link.select(&font_selector).next() {
                // 获取课程信息文本
                let text_lines: Vec<String> = font_tag
                    .text()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if text_lines.len() < 2 {
                    continue;
                }

                let course_name = text_lines[0].clone();
                let week_info = &text_lines[1];
                let teacher = if text_lines.len() > 2 {
                    text_lines[2].clone()
                } else {
                    "未指定".to_string()
                };
                let location = if text_lines.len() > 3 {
                    text_lines[3].clone()
                } else {
                    "未指定".to_string()
                };

                // 解析周次信息，如 "秋冬{第1-8周|2节/双周}"
                let (weeks_str, frequency) = parse_zju_week_info(week_info);

                // 检查 colspan 和 rowspan
                let colspan = td
                    .value()
                    .attr("colspan")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                let rowspan = td
                    .value()
                    .attr("rowspan")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);

                // 【关键】确定需要生成的单双周列表（严格按照 Python 逻辑）
                // Python: if colspan == 2 and '/周' in frequency and '/双周' not in frequency:
                let parity_list: Vec<&str> =
                    if colspan == 2 && frequency.contains("节/周") && !frequency.contains("双周") {
                        // 每周都上，生成单周和双周两条记录
                        vec!["单", "双"]
                    } else {
                        // 只按实际单双周生成
                        vec![parity_str]
                    };

                // 确定节次范围（处理 rowspan）
                let mut period_list: Vec<i32> = Vec::new();
                for r in 0..rowspan {
                    period_list.push(period + r as i32);
                }

                // 为每个(单双周, 节次)组合生成记录（Python: for p in parity_list: for per in period_list）
                for p in &parity_list {
                    for per in &period_list {
                        raw_courses.push(RawZJUCourse {
                            course_name: course_name.clone(),
                            weekday,
                            period: *per,
                            weeks: weeks_str.clone(),
                            parity: p.to_string(),
                            _frequency: frequency.clone(),
                            teacher: teacher.clone(),
                            location: location.clone(),
                        });
                    }
                }

                // 标记已处理的单元格
                processed_cells.insert(cell_id.to_string(), true);

                // 标记被合并的单元格（Python: merged_id = f"{base_parts[0]}-{int(base_parts[1]) + c}-{int(base_parts[2]) + r}"）
                if rowspan > 1 || colspan > 1 {
                    let base_parts: Vec<&str> = cell_id.split('-').collect();
                    if let (Ok(w), Ok(p_code)) = (
                        base_parts[0].parse::<i32>(),
                        base_parts[1].parse::<i32>(),
                    ) {
                        for r in 0..rowspan {
                            for c in 0..colspan {
                                if r == 0 && c == 0 {
                                    continue;
                                }
                                let merged_id = format!("{}-{}-{}", w, p_code + c as i32, period + r as i32);
                                processed_cells.insert(merged_id, true);
                            }
                        }
                    }
                }
            }
        }
    }

    // 合并记录并格式化（Python: merge_and_format）
    let merged_courses = merge_and_format_zju(raw_courses)?;

    Ok(merged_courses)
}

/// 浙江大学原始课程记录（对应 Python 脚本中的提取阶段）
#[derive(Debug, Clone)]
struct RawZJUCourse {
    course_name: String,
    weekday: i32,
    period: i32,
    weeks: String,
    parity: String,
    _frequency: String, // 保留用于逻辑判断，但不直接使用
    teacher: String,
    location: String,
}

/// 合并并格式化浙江大学课程（Python: merge_and_format）
fn merge_and_format_zju(raw_courses: Vec<RawZJUCourse>) -> Result<Vec<Course>, String> {
    // 第一步：按课程名称、星期、地点、教师分组（不含节次）
    // Python: key = (course['course_name'], course['weekday'], course['location'], course['teacher'])
    let mut course_groups: HashMap<String, Vec<&RawZJUCourse>> = HashMap::new();

    for course in &raw_courses {
        let key = format!(
            "{}|{}|{}|{}",
            course.course_name, course.weekday, course.location, course.teacher
        );

        course_groups
            .entry(key)
            .or_insert_with(Vec::new)
            .push(course);
    }

    // 第二步：对每个课程组进行处理
    let mut result = Vec::new();
    for (_key, courses) in course_groups.iter() {
        // 展开周数并合并（Python: groups[key]['weeks_list'].extend(WeekExpander.expand_weeks(...))）
        let mut weeks_list: Vec<i32> = Vec::new();
        for course in courses {
            let expanded = expand_zju_weeks(&course.weeks, &course.parity);
            weeks_list.extend(expanded);
        }
        weeks_list.sort();
        weeks_list.dedup();

        // 计算 week_type（如果周数是连续的则设为0=全周）
        let week_type = if weeks_list.len() > 1 {
            let is_continuous = weeks_list.windows(2).all(|w| w[1] == w[0] + 1);
            if is_continuous {
                0 // 全周
            } else {
                // 根据第一个课程的 parity 判断
                if courses[0].parity == "单" {
                    1
                } else {
                    2
                }
            }
        } else {
            0
        };

        // 【关键】查找相同课程的所有节次（Python: all_periods = [...]）
        let mut all_periods: Vec<i32> = courses.iter().map(|c| c.period).collect();
        all_periods.sort();
        all_periods.dedup();

        // 计算节次范围（Python: period_range）
        // Python: if len(all_periods) == 1: period_range = f"{all_periods[0]}-{all_periods[0] + 1}节"
        //       else: period_range = f"{all_periods[0]}-{all_periods[-1]}节"
        let periods = if all_periods.len() == 1 {
            // 单个节次，一节课通常是连续两小节
            vec![all_periods[0], all_periods[0] + 1]
        } else {
            // 多个节次，直接取最小到最大
            (all_periods[0]..=all_periods[all_periods.len() - 1]).collect()
        };

        let course = Course {
            name: courses[0].course_name.clone(),
            teacher: courses[0].teacher.clone(),
            weeks: weeks_list,
            week_type,
            day_of_week: courses[0].weekday,
            periods,
            location: courses[0].location.clone(),
        };
        result.push(course);
    }

    // 去重（Python: unique_result）
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut unique_result = Vec::new();
    for course in result {
        let key = format!(
            "{}|{}|{:?}|{}",
            course.day_of_week, course.name, course.periods, course.location
        );
        if !seen.contains(&key) {
            seen.insert(key);
            unique_result.push(course);
        }
    }

    Ok(unique_result)
}

/// 解析浙江大学周次信息
/// 输入如："秋冬{第1-8周|2节/双周}"
/// 返回：("第1-8周", "2节/双周")
fn parse_zju_week_info(text: &str) -> (String, String) {
    let week_re = Regex::new(r"第(\d+)-?(\d*)周").unwrap();
    let freq_re = Regex::new(r"(\d+)节/(周|双周)").unwrap();

    let weeks = week_re
        .captures(text)
        .map(|cap| {
            let start = cap.get(1).map(|m| m.as_str()).unwrap_or("1");
            let end = cap.get(2).map(|m| m.as_str()).filter(|s| !s.is_empty());
            match end {
                Some(e) => format!("第{}-{}周", start, e),
                None => format!("第{}周", start),
            }
        })
        .unwrap_or_default();

    let frequency = freq_re
        .captures(text)
        .map(|cap| {
            let count = cap.get(1).map(|m| m.as_str()).unwrap_or("2");
            let unit = cap.get(2).map(|m| m.as_str()).unwrap_or("周");
            format!("{}节/{}", count, unit)
        })
        .unwrap_or_default();

    (weeks, frequency)
}

/// 展开浙江大学周次字符串
/// 输入如："第1-8周"，parity="单"
/// 返回：[1, 3, 5, 7]
fn expand_zju_weeks(weeks_str: &str, parity: &str) -> Vec<i32> {
    let clean_str = weeks_str.replace("第", "").replace("周", "").trim().to_string();

    let (start, end) = if clean_str.contains('-') {
        let parts: Vec<&str> = clean_str.split('-').collect();
        let start_val = parts[0].parse().unwrap_or(1);
        let end_val = if parts.len() > 1 {
            parts[1].parse().unwrap_or(start_val)
        } else {
            start_val
        };
        (start_val, end_val)
    } else {
        let val = clean_str.parse().unwrap_or(1);
        (val, val)
    };

    let mut weeks = Vec::new();

    if parity == "单" {
        // 单周：1, 3, 5, 7...
        let mut w = if start % 2 == 1 { start } else { start + 1 };
        while w <= end {
            weeks.push(w);
            w += 2;
        }
    } else if parity == "双" {
        // 双周：2, 4, 6, 8...
        let mut w = if start % 2 == 0 { start } else { start + 1 };
        while w <= end {
            weeks.push(w);
            w += 2;
        }
    } else {
        // 全周
        for w in start..=end {
            weeks.push(w);
        }
    }

    weeks
}

/// 根据解析器类型解析 HTML
pub fn parse_html_with_parser(html_content: &str, parser_type: &str) -> Result<Vec<Course>, String> {
    match parser_type {
        "zju_default" => parse_zju_html(html_content),
        _ => parse_course_html(html_content), // 默认使用中央财经大学解析器
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_course_html() {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <body>
            <table class="timetable">
                <thead>
                    <tr><th colspan="9">Course Schedule</th></tr>
                    <tr><th>Section</th><th>Time</th><th>Mon</th><th>Tue</th><th>Wed</th><th>Thu</th><th>Fri</th><th>Sat</th><th>Sun</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td rowspan="4">上午</td>
                        <td>1</td>
                        <td>
                            <div class="timetable_con">
                                <span class="title">高等数学★</span>
                                <p title="教师">张三</p>
                                <p>1-16周</p>
                                <p title="上课地点">主教101</p>
                            </div>
                        </td>
                        <td>
                            <div class="timetable_con">
                                <span class="title">Test Course</span>
                                <p title="教师">Smith</p>
                                <p>2-4周</p>
                                <p title="上课地点">Room 202</p>
                            </div>
                        </td>
                        <td></td><td></td><td></td><td></td><td></td>
                    </tr>
                </tbody>
            </table>
        </body>
        </html>
        "#;

        let courses = parse_course_html(html).expect("Failed to parse");
        println!("Extracted courses: {:?}", courses);
        // Note: The parser logic for "Mon"/"Tue" depends on specific Chinese strings "周一", "周二" etc in header or assumed order.
        // My parser code: `let day_list = vec!["周一", "周二", ...];`
        // And it finds period columns based on skipping first 1 or 2 cols.
        // Wait, the parser logic iterates `cells` starting from index.
        // It assumes 3rd row is data.
        // And it maps columns to days using `col_rowspan` and `day_idx`.
        // BUT `day_list` is just a list of names to assign.
        // The parser logic does NOT look at the Table Header to determine which column is Monday.
        // It assumes standard layout: Period Col -> Mon -> Tue -> ...
        
        // In my mock HTML above, I have `<td>1</td>` (Period) then `<td>...` (Mon).
        // Let's verify parser logic:
        // `is_time_slot` check: "上午" is in first cell? Yes.
        // `start_cell_idx = 2`. Correct.
        // `cells.skip(2)` -> Mon, Tue...
        
        // So first course (Mon) should be parsed.
        // Second course (Tue) should be parsed.
        
        assert!(courses.len() >= 1);
        let c1 = &courses[0];
        assert_eq!(c1.name, "高等数学");
        assert_eq!(c1.day_of_week, 1); // Mon
        
        if courses.len() > 1 {
            let c2 = &courses[1];
            assert_eq!(c2.name, "Test Course");
            assert_eq!(c2.day_of_week, 2); // Tue
        }
    }
}
