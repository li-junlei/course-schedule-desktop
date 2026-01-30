<template>
  <div id="app" :style="{ backgroundImage: backgroundImage ? `url(${backgroundImage})` : 'none' }">
    <!-- 自定义导航栏 -->
    <div class="navbar">
      <div class="navbar-left" @click="showPopup = !showPopup">
        <el-icon :size="22"><MoreFilled /></el-icon>
      </div>
      <div class="navbar-center" @click="showWeekSelector = !showWeekSelector">
        <span>第 {{ currentWeek }} 周</span>
        <el-icon :size="14"><ArrowDown /></el-icon>
      </div>
    </div>

    <!-- 弹出菜单 -->
      <PopupMenu
      :show="showPopup"
      @close="showPopup = false"
      @schedule-manage="handleScheduleManage"
      @upload-bg="handleUploadBackground"
      @delete-bg="handleDeleteBackground"
      @settings="openSettings"
    />

    <!-- 加载状态 -->
    <div v-if="loading" class="loading">
      <el-icon class="is-loading" :size="20"><Loading /></el-icon>
      <span>正在从教务系统获取数据...</span>
    </div>

    <!-- 导入课表对话框 -->
    <el-dialog v-model="showImportDialog" title="导入课表" width="500px" :close-on-click-modal="false">
      <div style="margin-bottom: 20px;">
        <p style="margin-bottom: 10px;">请按照以下步骤获取课表数据：</p>
        <ol style="margin-left: 20px; line-height: 1.8;">
          <li>
            使用浏览器访问教务系统，登录并打开<b>【个人课表】</b>页面。
            <el-button type="primary" link @click="openBrowser">打开内置浏览器 (推荐)</el-button>
          </li>
          <li>
            在课表页面按 <kbd>F12</kbd> 打开开发者工具，找到 <code>&lt;html&gt;</code> 标签。
          </li>
          <li>右键点击 <code>&lt;html&gt;</code> 标签，选择 <b>Copy outerHTML</b>。</li>
          <li>或者：将页面另存为"网页，全部"，然后用文本编辑器打开HTML文件复制全部内容。</li>
          <li>将代码粘贴到下方输入框中。</li>
        </ol>
      </div>

      <el-form>
        <el-form-item label="课表名称" required>
          <el-input
            v-model="importScheduleName"
            placeholder="请输入课表名称，如：2024春季学期"
          />
        </el-form-item>
        <el-form-item label="">
          <el-input
            v-model="htmlSource"
            type="textarea"
            :rows="6"
            placeholder="在此处粘贴网页源代码..."
          />
        </el-form-item>
        <el-form-item>
             <el-button @click="readClipboard" style="width: 100%; margin-bottom: 10px;">
                 <el-icon><CopyDocument /></el-icon> 从剪贴板读取
             </el-button>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleImport" :loading="loading" style="width: 100%">
            解析并导入
          </el-button>
        </el-form-item>
      </el-form>
    </el-dialog>

    <!-- 设置对话框 -->
    <el-dialog v-model="showSettingsDialog" title="设置" width="500px">
      <el-form label-position="top">
        <el-form-item label="教务系统地址">
          <el-input
            v-model="tempConfig.edu_system_url"
            placeholder="例如：https://jwgl.xxx.edu.cn"
          />
          <div class="subtitle">填写后打开浏览器将直接跳转到该地址</div>
        </el-form-item>

        <el-form-item label="每日最大节次">
          <el-input-number v-model="tempConfig.max_periods" :min="8" :max="20" @change="handleMaxPeriodsChange" />
          <div class="subtitle">通常设为11或13</div>
        </el-form-item>

        <el-form-item label="节次时间设置">
            <div style="max-height: 300px; overflow-y: auto; width: 100%; border: 1px solid #eee; padding: 10px; border-radius: 4px;">
                <div v-for="(time, index) in tempPeriodTimes" :key="index" style="display: flex; align-items: center; margin-bottom: 8px;">
                    <span style="width: 50px; font-size: 13px;">第{{ index + 1 }}节</span>
                    <el-input v-model="time.start" placeholder="8:00" size="small" style="width: 100px;" />
                    <span style="margin: 0 8px;">-</span>
                    <el-input v-model="time.end" placeholder="8:45" size="small" style="width: 100px;" />
                </div>
            </div>
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="saveSettings" style="width: 100%">保存</el-button>
        </el-form-item>
      </el-form>
    </el-dialog>

    <!-- 课表管理对话框 -->
    <el-dialog v-model="showScheduleManageDialog" title="课表管理" width="500px">
      <div style="margin-bottom: 15px;">
        <el-button type="primary" @click="handleNewSchedule" style="width: 100%">
          <el-icon><Plus /></el-icon> 新建课表
        </el-button>
      </div>

      <div style="max-height: 400px; overflow-y: auto;">
        <el-empty v-if="scheduleList.length === 0" description="暂无课表" />
        <div
          v-for="schedule in scheduleList"
          :key="schedule.id"
          class="schedule-item"
          :class="{ active: schedule.id === currentScheduleId }"
        >
          <div class="schedule-info">
            <div class="schedule-name">{{ schedule.name }}</div>
            <div class="schedule-meta">
              {{ schedule.course_count }} 门课程 · {{ new Date(schedule.updated_at).toLocaleDateString() }} 更新
            </div>
          </div>
          <div class="schedule-actions">
            <el-button
              v-if="schedule.id !== currentScheduleId"
              type="primary"
              size="small"
              @click="handleSwitchSchedule(schedule.id)"
            >
              切换
            </el-button>
            <el-tag v-else type="success">当前</el-tag>
            <el-button
              type="danger"
              size="small"
              @click="handleDeleteSchedule(schedule.id)"
            >
              删除
            </el-button>
          </div>
        </div>
      </div>
    </el-dialog>

    <!-- 主内容 -->
    <div v-if="!showImportDialog" class="main-content">
      <!-- 周次选择器 -->
      <WeekSelector
        :show="showWeekSelector"
        :week="currentWeek"
        :courses="courses"
        @update:week="handleWeekChange"
      />

      <!-- 星期栏 -->
      <div class="week-bar">
        <div class="week-bar-month">
          <span>{{ getDate(1).split('/')[0] }}</span>
          <span>月</span>
        </div>
        <div v-for="(day, index) in weekDays" :key="index" class="week-bar-item">
          <span>{{ day }}</span>
          <span class="date">{{ getDate(index + 1) }}</span>
        </div>
      </div>

      <!-- 课表网格 -->
      <CourseGrid
        :courses="courses"
        :week="currentWeek"
        :end-week="endWeek"
        :colors="courseColors"
        :bg-image="backgroundImage"
        :max-periods="config.max_periods || 13"
        :period-times="config.period_times"
        @update:week="handleWeekChange"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import { MoreFilled, ArrowDown, Loading, CopyDocument, Plus } from '@element-plus/icons-vue';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { readFile } from '@tauri-apps/plugin-fs';
import { useCourse, useBrowserImport } from './composables/useCourse';
import { calculateDate } from './utils/date';
import { getShuffledColors } from './utils/color';
import PopupMenu from './components/PopupMenu.vue';
import WeekSelector from './components/WeekSelector.vue';
import CourseGrid from './components/CourseGrid.vue';
import type { AppConfig, ScheduleMetadata, PeriodTime } from './types';

// UI 状态
const showPopup = ref(false);
const showWeekSelector = ref(false);
const showImportDialog = ref(false);
const showSettingsDialog = ref(false);
const showScheduleManageDialog = ref(false);
const loading = ref(false);
const currentWeek = ref(1);
const endWeek = ref(20);

// 导入相关
const htmlSource = ref('');
const importScheduleName = ref('');
const isImportingFromBrowser = ref(false);

// 应用配置
const config = ref<AppConfig>({});
const tempConfig = ref<AppConfig>({}); // For editing in settings
const tempPeriodTimes = ref<PeriodTime[]>([]); // For editing time slots
const backgroundImage = ref('');

// 课表数据和颜色
const { courses, parseHtmlSchedule, loadCachedSchedule, saveScheduleCache, listSchedules, deleteSchedule, switchSchedule } = useCourse();
const { setupImportListener } = useBrowserImport();
const courseColors = getShuffledColors();

// 课表管理
const scheduleList = ref<ScheduleMetadata[]>([]);
const currentScheduleId = ref<string>();

const weekDays = ['周一', '周二', '周三', '周四', '周五', '周六', '周日'];

// 获取日期
function getDate(day: number): string {
  if (!config.value.first_day) {
    return '--/--';
  }
  return calculateDate(config.value.first_day, currentWeek.value, day);
}

// 处理周次变化
function handleWeekChange(week: number) {
  currentWeek.value = week;
  showWeekSelector.value = false;
}

// 读取剪贴板
async function readClipboard() {
    try {
        const text = await navigator.clipboard.readText();
        if (text) {
            htmlSource.value = text;
            ElMessage.success('已从剪贴板读取');
        } else {
            ElMessage.warning('剪贴板为空');
        }
    } catch (e) {
        ElMessage.error('无法读取剪贴板，请允许权限');
    }
}

// 打开内置浏览器
async function openBrowser() {
    try {
        // 使用配置中的教务系统URL，如果没有则使用百度搜索
        const url = config.value.edu_system_url || 'https://www.baidu.com/s?wd=教务系统';
        await invoke('open_login_window', { url });
        ElMessage.info('请在打开的窗口中登录教务系统，并在课表页面点击右下角的导入按钮');
    } catch (e) {
        console.error(e);
        ElMessage.error(`打开浏览器失败: ${e}`);
    }
}

// 处理导入
async function handleImport() {
  if (!htmlSource.value) {
    ElMessage.warning('请粘贴HTML源代码');
    return;
  }

  if (!importScheduleName.value) {
    ElMessage.warning('请输入课表名称');
    return;
  }

  loading.value = true;
  try {
    await parseHtmlSchedule(htmlSource.value);
    // 保存课表
    await saveScheduleCache(courses.value, importScheduleName.value);
    showImportDialog.value = false;
    htmlSource.value = '';
    importScheduleName.value = '';
    ElMessage.success('导入成功');
    loadConfig();
    await loadScheduleList();
  } catch (e) {
    ElMessage.error(`导入失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

// 处理课表管理
async function handleScheduleManage() {
  showPopup.value = false;
  showScheduleManageDialog.value = true;
  await loadScheduleList();
}

// 加载课表列表
async function loadScheduleList() {
  try {
    scheduleList.value = await listSchedules();
    const cfg = await invoke<AppConfig>('get_app_config');
    currentScheduleId.value = cfg.current_schedule_id;
  } catch (e) {
    console.error('加载课表列表失败:', e);
  }
}

// 切换课表
async function handleSwitchSchedule(scheduleId: string) {
  try {
    await switchSchedule(scheduleId);
    await loadCachedSchedule(scheduleId);
    currentScheduleId.value = scheduleId;
    showScheduleManageDialog.value = false;
    ElMessage.success('切换成功');
    await loadConfig();
  } catch (e) {
    ElMessage.error(`切换失败: ${e}`);
  }
}

// 删除课表
async function handleDeleteSchedule(scheduleId: string) {
  try {
    await deleteSchedule(scheduleId);
    ElMessage.success('删除成功');
    await loadScheduleList();
    // 如果删除的是当前课表,清空显示
    if (currentScheduleId.value === scheduleId) {
      courses.value = [];
    }
  } catch (e) {
    ElMessage.error(`删除失败: ${e}`);
  }
}

// 新建课表
function handleNewSchedule() {
  showScheduleManageDialog.value = false;
  importScheduleName.value = '';
  htmlSource.value = '';
  showImportDialog.value = true;
}

// 处理背景上传
async function handleUploadBackground() {
  showPopup.value = false;
  
  try {
    const selected = await openDialog({
      multiple: false,
      filters: [{
        name: '图片',
        extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp']
      }]
    });

    if (selected) {
      // 读取图片文件并转换为base64
      const fileData = await readFile(selected as string);
      const base64 = btoa(
        new Uint8Array(fileData).reduce((data, byte) => data + String.fromCharCode(byte), '')
      );
      
      // 获取文件扩展名
      const ext = (selected as string).split('.').pop()?.toLowerCase() || 'png';
      const mimeType = ext === 'jpg' ? 'image/jpeg' : `image/${ext}`;
      
      backgroundImage.value = `data:${mimeType};base64,${base64}`;
      
      // 保存配置
      await invoke('save_background_image', { sourcePath: selected });
      config.value.background_image = selected as string;
      
      ElMessage.success('背景设置成功');
    }
  } catch (e) {
    console.error('选择背景失败:', e);
    ElMessage.error(`设置背景失败: ${e}`);
  }
}

// 处理删除背景
async function handleDeleteBackground() {
  showPopup.value = false;
  
  try {
    await invoke('delete_background_image');
    config.value.background_image = undefined;
    backgroundImage.value = '';
    ElMessage.success('背景已删除');
  } catch (e) {
    ElMessage.error(`删除失败: ${e}`);
  }
}

// 打开设置
function openSettings() {
  showPopup.value = false;
  tempConfig.value = { ...config.value, max_periods: config.value.max_periods || 13 };

  // Initialize times
  const max = tempConfig.value.max_periods || 13;
  const currentTimes = config.value.period_times || [];
  const times: PeriodTime[] = [];

  for (let i = 0; i < max; i++) {
    if (currentTimes[i]) {
      times.push(currentTimes[i]);
    } else {
      // Default generation
      const p = i + 1;
      let startH = 8 + p - 1;
      if (p > 4) startH += 2;
      if (p > 8) startH += 1;

      // 计算结束时间(默认每节课45分钟)
      const endH = startH;
      const endM = 45;

      times.push({
        start: `${startH}:00`,
        end: `${endH}:${endM < 10 ? '0' + endM : endM}`
      });
    }
  }
  tempPeriodTimes.value = times;

  showSettingsDialog.value = true;
}

// Handle max periods change to resize array
function handleMaxPeriodsChange(val: number) {
    const oldTimes = [...tempPeriodTimes.value];
    const newTimes: PeriodTime[] = [];
    for (let i = 0; i < val; i++) {
        if (oldTimes[i]) {
            newTimes.push(oldTimes[i]);
        } else {
             // Default generation
             const p = i + 1;
             let startH = 8 + p - 1;
             if (p > 4) startH += 2;
             if (p > 8) startH += 1;

             const endH = startH;
             const endM = 45;

             newTimes.push({
               start: `${startH}:00`,
               end: `${endH}:${endM < 10 ? '0' + endM : endM}`
             });
        }
    }
    tempPeriodTimes.value = newTimes;
}

// 保存设置
async function saveSettings() {
  try {
    const newConfig = {
        ...config.value,
        ...tempConfig.value,
        period_times: tempPeriodTimes.value
    };
    await invoke('save_app_config', { config: newConfig });
    config.value = newConfig;
    showSettingsDialog.value = false;
    ElMessage.success('设置保存成功');
  } catch (e) {
    ElMessage.error(`保存失败: ${e}`);
  }
}

// 加载配置
async function loadConfig() {
  try {
    const appConfig = await invoke<AppConfig>('get_app_config');
    config.value = appConfig;

    // 计算当前周次
    if (appConfig.first_day) {
      const week = Math.floor((Date.now() - appConfig.first_day) / (7 * 24 * 60 * 60 * 1000)) + 1;
      currentWeek.value = Math.max(Math.min(week, 20), 1);
    }

    // 加载背景图
    if (appConfig.background_image) {
      try {
        const fileData = await readFile(appConfig.background_image);
        const base64 = btoa(
          new Uint8Array(fileData).reduce((data, byte) => data + String.fromCharCode(byte), '')
        );
        const ext = appConfig.background_image.split('.').pop()?.toLowerCase() || 'png';
        const mimeType = ext === 'jpg' ? 'image/jpeg' : `image/${ext}`;
        backgroundImage.value = `data:${mimeType};base64,${base64}`;
      } catch (e) {
        console.error('加载背景图失败:', e);
      }
    }

    // 设置结束周次
    if (appConfig.end_week) {
      endWeek.value = appConfig.end_week;
    }
  } catch (e) {
    console.error('加载配置失败:', e);
  }
}

// 初始化
onMounted(async () => {
  // 设置浏览器导入监听
  setupImportListener(async (importedCourses) => {
    courses.value = importedCourses;
    loading.value = false;
    isImportingFromBrowser.value = false;

    // 提示用户输入课表名称
    showImportDialog.value = true;
    importScheduleName.value = '从浏览器导入 ' + new Date().toLocaleDateString();
  });

  // 尝试加载缓存
  const hasCache = await loadCachedSchedule();

  if (hasCache) {
    await loadConfig();
  } else {
    // 如果没有缓存,直接打开浏览器
    isImportingFromBrowser.value = true;
    openBrowser();
  }
});
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

#app {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background-color: #f5f5f5;
  background-size: cover;
  background-position: center;
}

/* 导航栏 */
.navbar {
  height: 50px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  background-color: rgba(255, 255, 255, 0.9);
}

.navbar-left {
  position: absolute;
  left: 15px;
  cursor: pointer;
  padding: 8px;
  border-radius: 50%;
  transition: background-color 0.2s;
}

.navbar-left:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

.navbar-center {
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  font-size: 15px;
  font-weight: 500;
  padding: 6px 12px;
  border-radius: 16px;
  transition: background-color 0.2s;
}

.navbar-center:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

/* 加载状态 */
.loading {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 24px;
  background-color: white;
  border-radius: 20px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  font-size: 14px;
}

/* 主内容 */
.main-content {
  height: calc(100vh - 50px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 星期栏 */
.week-bar {
  display: flex;
  height: 36px;
  background-color: rgba(255, 255, 255, 0.9);
  border-bottom: 1px solid #e0e0e0;
  flex-shrink: 0;
}

.week-bar-month {
  width: 6%;
  min-width: 45px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-size: 12px;
}

.week-bar-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-size: 12px;
}

.week-bar-item .date {
  color: #999;
  font-size: 10px;
}

/* 深色模式 */
@media (prefers-color-scheme: dark) {
  #app {
    background-color: #1a1a1a;
    color: #fff;
  }

  .navbar {
    background-color: rgba(30, 30, 30, 0.9);
  }

  .week-bar {
    background-color: rgba(30, 30, 30, 0.9);
    border-bottom-color: #333;
  }

  .loading {
    background-color: #2a2a2a;
  }

  .schedule-item {
    background-color: #2a2a2a;
    border-color: #333;
  }

  .schedule-item:hover {
    background-color: #333;
  }

  .schedule-item.active {
    background-color: rgba(103, 194, 58, 0.1);
    border-color: #67C23A;
  }
}

/* 课表管理样式 */
.schedule-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 15px;
  margin-bottom: 10px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  background-color: white;
  transition: all 0.3s;
}

.schedule-item:hover {
  background-color: #f5f5f5;
}

.schedule-item.active {
  background-color: rgba(103, 194, 58, 0.1);
  border-color: #67C23A;
}

.schedule-info {
  flex: 1;
}

.schedule-name {
  font-size: 16px;
  font-weight: 500;
  margin-bottom: 5px;
}

.schedule-meta {
  font-size: 12px;
  color: #999;
}

.schedule-actions {
  display: flex;
  gap: 8px;
}

.subtitle {
  font-size: 12px;
  color: #999;
  margin-top: 4px;
}
</style>
