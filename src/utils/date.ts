/**
 * 格式化日期为 M/D 格式
 * @param date - 日期对象
 * @returns 格式化后的日期字符串
 */
export function formatDate(date: Date): string {
  const month = date.getMonth() + 1;
  const day = date.getDate();
  return `${month}/${day}`;
}

/**
 * 计算指定周次和星期的日期
 * @param firstDay - 学期第一天的时间戳
 * @param targetWeek - 目标周次
 * @param targetDay - 目标星期 (1-7)
 * @returns 格式化后的日期字符串
 */
export function calculateDate(firstDay: number, targetWeek: number, targetDay: number): string {
  const firstDate = new Date(firstDay);
  const targetDate = new Date(firstDate);

  // 计算目标日期
  targetDate.setDate(firstDate.getDate() + (targetWeek - 1) * 7 + (targetDay - 1));

  return formatDate(targetDate);
}

/**
 * 获取当前周次
 * @param firstDay - 学期第一天的时间戳
 * @returns 当前周次
 */
export function getCurrentWeek(firstDay: number): number {
  const now = Date.now();
  const week = Math.floor((now - firstDay) / (7 * 24 * 60 * 60 * 1000)) + 1;
  return Math.max(week, 1);
}
