/**
 * 课程数据结构
 */
export interface Course {
  /** 课程名称 */
  name: string;
  /** 教师名称 */
  teacher: string;
  /** 周次范围 [开始周, 结束周, 开始周2(可选), 结束周2(可选)] */
  weeks: number[];
  /** 单双周标记 (1=单周, 2=双周, 0=全周) */
  week_type: number;
  /** 星期 (1-7) */
  day_of_week: number;
  /** 节次范围 [开始节, 结束节] */
  periods: number[];
  /** 教室/地点 */
  location: string;
}

/**
 * 时间段(包含开始和结束时间)
 */
export interface PeriodTime {
  /** 开始时间，格式: "8:00" */
  start: string;
  /** 结束时间，格式: "8:45" */
  end: string;
}

/**
 * 应用配置
 */
export interface AppConfig {
  /** 背景图片文件名 */
  background_image?: string;
  /** 学期第一天的时间戳 */
  first_day?: number;
  /** 学期结束周次 */
  end_week?: number;
  /** 最大节次 (默认13) */
  max_periods?: number;
  /** 节次时间表 */
  period_times?: PeriodTime[];
  /** 教务系统地址 */
  edu_system_url?: string;
  /** 当前选中的课表ID */
  current_schedule_id?: string;
}

/**
 * 课表元数据
 */
export interface ScheduleMetadata {
  /** 课表唯一ID */
  id: string;
  /** 课表名称 */
  name: string;
  /** 创建时间 */
  created_at: number;
  /** 更新时间 */
  updated_at: number;
  /** 课程数量 */
  course_count: number;
}
