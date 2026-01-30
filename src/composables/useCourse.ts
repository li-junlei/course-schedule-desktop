import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Course, ScheduleMetadata } from '../types';

/**
 * 课表数据管理 Composable
 */
export function useCourse() {
  const courses = ref<Course[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  /**
   * 登录并获取课表
   */
  async function loginAndGetSchedule(
    username: string,
    password: string,
    baseUrl: string
  ): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      const result = await invoke<Course[]>('login_and_get_schedule', {
        username,
        password,
        baseUrl,
      });
      courses.value = result;
      // 保存到缓存
      await saveScheduleCache(result, '未命名课表');
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 刷新课表
   */
  async function refreshSchedule(baseUrl: string): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      const result = await invoke<Course[]>('refresh_schedule', { baseUrl });
      courses.value = result;
      await saveScheduleCache(result, '未命名课表');
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 加载缓存的课表
   */
  async function loadCachedSchedule(scheduleId?: string): Promise<boolean> {
    try {
      const result = await invoke<Course[]>('load_cached_schedule', {
        scheduleId
      });
      courses.value = result;
      return true;
    } catch (e) {
      // 缓存不存在或已过期
      return false;
    }
  }

  /**
   * 保存课表到缓存
   */
  async function saveScheduleCache(coursesData: Course[], name: string, scheduleId?: string): Promise<string> {
    try {
      const id = await invoke<string>('save_schedule_cache', {
        courses: coursesData,
        name,
        scheduleId
      });
      return id;
    } catch (e) {
      console.error('保存缓存失败:', e);
      throw e;
    }
  }

  /**
   * 解析 HTML 获取课表
   */
  async function parseHtmlSchedule(html: string): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      const result = await invoke<Course[]>('parse_html_schedule', { html });
      courses.value = result;
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 获取所有课表列表
   */
  async function listSchedules(): Promise<ScheduleMetadata[]> {
    try {
      return await invoke<ScheduleMetadata[]>('list_schedules');
    } catch (e) {
      console.error('获取课表列表失败:', e);
      throw e;
    }
  }

  /**
   * 删除指定课表
   */
  async function deleteSchedule(scheduleId: string): Promise<void> {
    try {
      await invoke('delete_schedule', { scheduleId });
    } catch (e) {
      console.error('删除课表失败:', e);
      throw e;
    }
  }

  /**
   * 切换当前课表
   */
  async function switchSchedule(scheduleId: string): Promise<void> {
    try {
      await invoke('switch_schedule', { scheduleId });
    } catch (e) {
      console.error('切换课表失败:', e);
      throw e;
    }
  }

  /**
   * 计算最大周次
   */
  const maxWeek = computed(() => {
    let max = 0;
    for (const course of courses.value) {
      if (course.weeks && course.weeks.length > 0) {
        // rust端的weeks已经是展开的周次列表了，直接取最大值
        const courseMax = Math.max(...course.weeks);
        max = Math.max(max, courseMax);
      }
    }
    return max || 20;
  });

  return {
    courses,
    loading,
    error,
    loginAndGetSchedule,
    parseHtmlSchedule,
    refreshSchedule,
    loadCachedSchedule,
    saveScheduleCache,
    listSchedules,
    deleteSchedule,
    switchSchedule,
    maxWeek,
  };
}

/**
 * 监听浏览器导入事件
 */
export function useBrowserImport() {
  const courses = ref<Course[]>([]);
  const isImporting = ref(false);

  // 监听来自浏览器的导入事件
  async function setupImportListener(callback: (courses: Course[]) => void) {
    try {
      const unlisten = await listen<Course[]>('schedule-imported', (event) => {
        courses.value = event.payload;
        callback(event.payload);
      });
      return unlisten;
    } catch (e) {
      console.error('设置导入监听器失败:', e);
      throw e;
    }
  }

  return {
    courses,
    isImporting,
    setupImportListener,
  };
}
