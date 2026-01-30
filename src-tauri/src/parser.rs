use scraper::{Html, Selector, ElementRef};
use regex::Regex;
use std::collections::HashMap;
use crate::models::Course;

/// 解析HTML课程表
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
