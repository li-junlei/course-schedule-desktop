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
      @import-schedule="handleNewSchedule"
      @appearance="showAppearanceDialog = true"
      @settings="openSettings"
    />

    <!-- 加载状态 -->
    <div v-if="loading" class="loading">
      <el-icon class="is-loading" :size="20"><Loading /></el-icon>
      <span>正在从教务系统获取数据...</span>
    </div>

    <!-- 导入课表对话框 -->
    <el-dialog
      v-model="showImportDialog"
      title=""
      width="90%"
      align-center
      :show-close="false"
      class="custom-dialog"
      append-to-body
      style="max-width: 480px;"
    >
      <div class="dialog-header">
        <div class="dialog-title">导入课表</div>
        <div class="dialog-close-btn" @click="showImportDialog = false">
          <el-icon :size="20"><Close /></el-icon>
        </div>
      </div>
      <div class="dialog-content">
        <div class="setting-item">
            <div class="setting-label">选择解析脚本</div>
             <el-select v-model="selectedParserConfig" placeholder="请选择脚本" class="modern-input" style="width: 100%">
               <el-option
                 v-for="config in parserConfigs"
                 :key="config.id"
                 :label="config.name"
                 :value="config.id"
               />
             </el-select>
        </div>
        <div class="file-upload-area" @click="handleFileSelect" @drop.prevent="handleFileDrop" @dragover.prevent>
             <el-icon :size="48" color="var(--primary-color)"><Upload /></el-icon>
             <div class="upload-text">点击或拖拽 HTML 文件至此</div>
        </div>
      </div>
      <div class="dialog-footer">
        <el-button @click="showImportDialog = false" class="modern-button">取消</el-button>
      </div>
    </el-dialog>

    <!-- 设置对话框 -->
    <el-dialog
      v-model="showSettingsDialog"
      title=""
      width="90%"
      align-center
      :show-close="false"
      class="custom-dialog"
      append-to-body
      style="max-width: 460px;"
    >
      <div class="dialog-header">
        <div class="dialog-title">设置</div>
        <div class="dialog-close-btn" @click="showSettingsDialog = false">
          <el-icon :size="20"><Close /></el-icon>
        </div>
      </div>

      <div class="dialog-content no-scrollbar">
        <div class="setting-section">
          <div class="section-title">教务系统</div>
          <div class="setting-item">
            <div class="setting-label">系统地址</div>
            <el-input
              v-model="tempConfig.edu_system_url"
              placeholder="例如：https://jwgl.xxx.edu.cn"
              class="modern-input"
            >
              <template #prefix>
                <el-icon><Link /></el-icon>
              </template>
            </el-input>
            <div class="setting-desc">填写后打开浏览器将直接跳转到该地址</div>
          </div>
        </div>

        <div class="setting-section">
          <div class="section-title">课表显示</div>
          <div class="setting-item">
            <div class="setting-row">
              <div class="setting-label">每日节数</div>
              <el-input-number
                v-model="tempConfig.max_periods"
                :min="8"
                :max="20"
                @change="handleMaxPeriodsChange"
                class="modern-number-input"
                controls-position="right"
              />
            </div>
            <div class="setting-desc">设置每天显示的最大课程节数</div>
          </div>

          <div class="setting-item">
            <div class="setting-label">节次时间</div>
            <div class="time-slots-container">
              <div v-for="(time, index) in tempPeriodTimes" :key="index" class="time-slot-row">
                <div class="slot-index">{{ index + 1 }}</div>
                <div class="slot-inputs">
                  <el-input v-model="time.start" placeholder="08:00" class="time-mini-input" />
                  <span class="time-divider"></span>
                  <el-input v-model="time.end" placeholder="08:45" class="time-mini-input" />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="dialog-footer">
        <el-button @click="showSettingsDialog = false" class="modern-button">取消</el-button>
        <el-button type="primary" @click="saveSettings" class="modern-button primary">
          保存更改
        </el-button>
      </div>
    </el-dialog>

    <!-- 课表外观对话框 -->
    <el-dialog
      v-model="showAppearanceDialog"
      title=""
      width="90%"
      align-center
      :show-close="false"
      class="custom-dialog"
      append-to-body
      style="max-width: 500px;"
    >
      <div class="dialog-header">
        <div class="dialog-title">外观设置</div>
        <div class="dialog-close-btn" @click="showAppearanceDialog = false">
          <el-icon :size="20"><Close /></el-icon>
        </div>
      </div>

      <div class="dialog-content">
        <div class="appearance-grid">
          <div class="appearance-item" @click="handleUploadBackground">
            <div class="item-preview bg-preview">
               <el-image 
                 v-if="backgroundImage" 
                 :src="backgroundImage" 
                 fit="cover" 
                 style="width: 100%; height: 100%;"
               />
               <div v-else class="placeholder-icon">
                 <el-icon :size="32"><Picture /></el-icon>
               </div>
               <div class="hover-overlay">
                 <el-icon><Upload /></el-icon>
                 <span>更换背景</span>
               </div>
            </div>
            <div class="item-info">
              <div class="item-title">自定义背景图</div>
              <div class="item-desc">支持 JPG/PNG/WEBP</div>
            </div>
          </div>

          <div class="appearance-item" :class="{ disabled: !backgroundImage }" @click="backgroundImage && handleDeleteBackground()">
            <div class="item-preview delete-preview">
               <el-icon :size="32"><Delete /></el-icon>
            </div>
            <div class="item-info">
              <div class="item-title">移除背景</div>
              <div class="item-desc">恢复默认纯净背景</div>
            </div>
          </div>
        </div>
      </div>
    </el-dialog>

    <!-- 课表管理对话框 -->
    <el-dialog
      v-model="showScheduleManageDialog"
      title=""
      width="90%"
      align-center
      :show-close="false"
      class="custom-dialog"
      append-to-body
      style="max-width: 520px;"
    >
      <div class="dialog-header">
        <div class="dialog-title">课表管理</div>
        <div class="dialog-close-btn" @click="showScheduleManageDialog = false">
          <el-icon :size="20"><Close /></el-icon>
        </div>
      </div>

      <div class="dialog-content no-scrollbar">
        <div class="schedule-list-container">
            <div 
              v-for="schedule in scheduleList"
              :key="schedule.id"
              class="schedule-list-item"
              :class="{ 'is-active': schedule.id === currentScheduleId }"
              @click="schedule.id !== currentScheduleId && handleSwitchSchedule(schedule.id)"
            >
              <div class="item-main">
                <div class="item-header">
                  <span class="item-name">{{ schedule.name }}</span>
                  <el-tag v-if="schedule.id === currentScheduleId" size="small" effect="dark" round class="status-tag">当前使用</el-tag>
                </div>
                <div class="item-meta">
                  <span><el-icon><Collection /></el-icon> {{ schedule.course_count }} 门课</span>
                  <span><el-icon><Clock /></el-icon> {{ new Date(schedule.updated_at * 1000).toLocaleDateString() }}</span>
                </div>
              </div>
              
              <div class="item-actions">
                <el-button 
                  circle 
                  text 
                  @click.stop="handleEditScheduleDate(schedule)"
                  class="action-btn"
                  title="设置起始日期"
                >
                  <el-icon><Calendar /></el-icon>
                </el-button>
                <el-button 
                  circle 
                  text 
                  type="danger" 
                  @click.stop="handleDeleteSchedule(schedule.id)"
                  class="action-btn danger"
                  title="删除课表"
                >
                  <el-icon><Delete /></el-icon>
                </el-button>
              </div>
            </div>
            
            <div v-if="scheduleList.length === 0" class="empty-state">
                <el-icon :size="48" class="empty-icon"><Collection /></el-icon>
                <p>暂无课表数据</p>
            </div>
        </div>
      </div>
      
      <div class="dialog-footer">
          <el-button type="primary" class="modern-button primary full-width" @click="handleNewSchedule">
            <el-icon style="margin-right: 6px;"><Plus /></el-icon> 新建 / 导入课表
          </el-button>
      </div>
    </el-dialog>

    <!-- 设置课表日期对话框 -->
    <el-dialog
      v-model="showScheduleDateDialog"
      title=""
      width="90%"
      align-center
      :show-close="false"
      class="custom-dialog"
      append-to-body
      style="max-width: 400px;"
    >
      <div class="dialog-header">
        <div class="dialog-title">设置日期</div>
        <div class="dialog-close-btn" @click="showScheduleDateDialog = false">
          <el-icon :size="20"><Close /></el-icon>
        </div>
      </div>

      <div class="dialog-content">
        <div class="setting-item">
          <div class="setting-label">课表名称</div>
          <el-input 
            v-model="editingSchedule.name" 
            disabled 
            class="modern-input"
          >
            <template #prefix><el-icon><Collection /></el-icon></template>
          </el-input>
        </div>

        <div class="setting-item">
          <div class="setting-label">第一周起始日</div>
          <el-date-picker
            v-model="editingSchedule.first_day_date"
            type="date"
            placeholder="选择第一周周一的日期"
            format="YYYY/MM/DD"
            value-format="YYYY-MM-DD"
            style="width: 100%"
            class="modern-input"
            :clearable="false"
          />
          <div class="setting-desc">校历第一周的周一日期，用于计算当前周次</div>
        </div>
      </div>

      <div class="dialog-footer">
        <el-button @click="showScheduleDateDialog = false" class="modern-button">取消</el-button>
        <el-button type="primary" @click="handleSaveScheduleDate" class="modern-button primary">
          保存设置
        </el-button>
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
        @course-click="handleCourseClick"
      />

    <!-- 课程详情底部抽屉 -->
    <Transition name="slide-up">
      <div v-if="showDetailSheet && selectedCourse" class="detail-sheet-overlay" @click="showDetailSheet = false">
        <div class="detail-sheet" @click.stop>
          <div class="detail-handle"></div>
          
          <div class="detail-header">
            <div class="detail-name">{{ selectedCourse.name }}</div>
          </div>

          <div class="detail-content">
            <!-- Row 2: Weeks -->
            <div class="detail-row">
              <div class="detail-icon-box blue">
                <el-icon><Calendar /></el-icon>
              </div>
              <div class="detail-info">
                <span class="detail-label">周次</span>
                <span class="detail-value">{{ getWeeksText(selectedCourse.weeks) }}</span>
              </div>
            </div>

            <!-- Row 3: Time -->
            <div class="detail-row">
              <div class="detail-icon-box green">
                <el-icon><Timer /></el-icon>
              </div>
              <div class="detail-info">
                <span class="detail-label">时间</span>
                <span class="detail-value">{{ getCourseTimeText(selectedCourse) }}</span>
              </div>
            </div>

            <!-- Row 4: Teacher -->
            <div class="detail-row">
              <div class="detail-icon-box orange">
                <el-icon><User /></el-icon>
              </div>
              <div class="detail-info">
                <span class="detail-label">教师</span>
                <span class="detail-value">{{ selectedCourse.teacher || '未设置' }}</span>
              </div>
            </div>

            <!-- Row 5: Location -->
            <div class="detail-row">
              <div class="detail-icon-box purple">
                <el-icon><Location /></el-icon>
              </div>
              <div class="detail-info">
                <span class="detail-label">教室</span>
                <span class="detail-value">{{ selectedCourse.location || '未设置' }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    </div>

    <!-- 主题切换按钮 -->
    <div class="theme-toggle" @click="toggleTheme" :title="isDark ? '切换亮色模式' : '切换深色模式'">
      <el-icon :size="20">
        <Moon v-if="!isDark" />
        <Sunny v-else />
      </el-icon>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { MoreFilled, ArrowDown, Loading, CopyDocument, Plus, Picture, Delete, Close, Clock, Calendar, Warning, Collection, Sunny, Moon, Link, Upload, Timer, User, Location } from '@element-plus/icons-vue';
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
const showAppearanceDialog = ref(false);
const showScheduleManageDialog = ref(false);
const showScheduleDateDialog = ref(false);
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

// 编辑课表
const editingSchedule = ref<{
  id: string;
  name: string;
  first_day?: number;
  first_day_date?: string;
}>({
  id: '',
  name: '',
  first_day: undefined,
  first_day_date: undefined,
});

const weekDays = ['周一', '周二', '周三', '周四', '周五', '周六', '周日'];

// 获取日期
function getDate(day: number): string {
  if (!config.value.first_day) {
    return '--/--';
  }
  return calculateDate(config.value.first_day, currentWeek.value, day);
}

// 详情抽屉状态
const showDetailSheet = ref(false);
const selectedCourse = ref<any | null>(null);

function handleCourseClick(course: any) {
  console.log('App received course click:', course?.name);
  selectedCourse.value = course;
  showDetailSheet.value = true;
  console.log('Sheet visibility:', showDetailSheet.value);
}

function getWeeksText(weeks: number[]) {
  if (weeks.length === 0) return '无';
  
  // Sort and unique
  const sorted = [...new Set(weeks)].sort((a, b) => a - b);
  
  const ranges: string[] = [];
  let start = sorted[0];
  let prev = sorted[0];
  
  for (let i = 1; i < sorted.length; i++) {
    const current = sorted[i];
    if (current !== prev + 1) {
      // End of a range
      if (start === prev) {
         ranges.push(`${start}`);
      } else {
         ranges.push(`${start}-${prev}`);
      }
      start = current;
    }
    prev = current;
  }
  
  // Handle final range
  if (start === prev) {
    ranges.push(`${start}`);
  } else {
    ranges.push(`${start}-${prev}`);
  }
  
  return ranges.join(', ') + ' 周';
}

function getCourseTimeText(course: any) {
  const startPeriod = course.periods[0];
  const endPeriod = course.periods[course.periods.length - 1];
  
  const startTime = getPeriodTime(startPeriod, true);
  const endTime = getPeriodTime(endPeriod, false);
  
  return `第 ${startPeriod}-${endPeriod} 节  ${startTime}-${endTime}`;
}

function getPeriodTime(period: number, isStart: boolean) {
  // Use saved times if available, else calc default
  if (tempPeriodTimes.value[period - 1]) {
    return isStart ? tempPeriodTimes.value[period - 1].start : tempPeriodTimes.value[period - 1].end;
  }
  // Default fallback calculation matches CourseGrid logic (simplified)
  const p = period;
  const startHour = 8 + Math.floor((p - 1) * 55 / 60);
  const startMin = ((p - 1) * 55) % 60;
  const endMin = (startMin + 45) % 60;
  const endHour = startHour + Math.floor((startMin + 45) / 60);
  
  if (isStart) 
    return `${startHour}:${startMin.toString().padStart(2, '0')}`;
  else
    return `${endHour}:${endMin.toString().padStart(2, '0')}`;
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
        // 先让用户输入课表名称
        const { value } = await ElMessageBox.prompt('请输入课表名称：', '导入课表', {
            confirmButtonText: '确定',
            cancelButtonText: '取消',
            inputValue: `从浏览器导入 ${new Date().toLocaleDateString()}`,
            inputPattern: /\S+/,
            inputErrorMessage: '课表名称不能为空'
        }).catch(() => {
            // 用户取消
            return { value: null };
        });

        if (!value) {
            return; // 用户取消了
        }

        // 保存课表名称
        importScheduleName.value = value;

        // 使用配置中的教务系统URL，如果没有则使用百度搜索
        const url = config.value.edu_system_url || 'https://www.baidu.com/s?wd=教务系统';
        await invoke('open_login_window', { url });
        ElMessage.info('请在打开的窗口中登录教务系统，并在课表页面点击右下角的导入按钮');

        // 监听窗口关闭事件（轮询方式）
        const checkInterval = setInterval(async () => {
            try {
                // 尝试获取 login_window，如果不存在会抛出错误
                await invoke('get_window_state', { windowLabel: 'login_window' });
            } catch (e) {
                // 窗口不存在，说明已关闭
                clearInterval(checkInterval);
                console.log('浏览器窗口已关闭，尝试从剪贴板导入...');

                // 从剪贴板读取并导入
                try {
                    const text = await navigator.clipboard.readText();
                    if (text && text.length > 1000) { // HTML 内容通常很长
                        console.log('从剪贴板读取到 HTML，长度:', text.length);

                        // 使用用户输入的名称导入
                        loading.value = true;
                        try {
                            await parseHtmlSchedule(text);
                            await saveScheduleCache(courses.value, importScheduleName.value);
                            ElMessage.success('课表导入成功！');
                            await loadScheduleList();
                            htmlSource.value = '';
                        } catch (err) {
                            ElMessage.error(`导入失败: ${err}`);
                        } finally {
                            loading.value = false;
                        }
                    } else {
                        console.log('剪贴板为空或内容太短，忽略');
                    }
                } catch (clipErr) {
                    console.error('读取剪贴板失败:', clipErr);
                    ElMessage.error('读取剪贴板失败，请确保已授予剪贴板权限');
                }
            }
        }, 1000); // 每秒检查一次

        // 5分钟后停止检查（避免无限轮询）
        setTimeout(() => clearInterval(checkInterval), 300000);
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
    const schedules = await listSchedules();
    console.log('加载到的课表列表:', schedules);
    scheduleList.value = schedules;

    const cfg = await invoke<AppConfig>('get_app_config');
    currentScheduleId.value = cfg.current_schedule_id;

    console.log('当前课表ID:', currentScheduleId.value);
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
async function handleNewSchedule() {
  showScheduleManageDialog.value = false;
  // 直接打开浏览器，不需要显示导入对话框
  await openBrowser();
}

// 编辑课表日期
async function handleEditScheduleDate(schedule: ScheduleMetadata) {
  const firstDayDate = schedule.first_day
    ? new Date(schedule.first_day * 1000).toISOString().split('T')[0]
    : undefined;

  console.log('编辑课表日期:', {
    schedule,
    first_day_timestamp: schedule.first_day,
    first_day_date: firstDayDate
  });

  editingSchedule.value = {
    id: schedule.id,
    name: schedule.name,
    first_day: schedule.first_day,
    first_day_date: firstDayDate,
  };
  showScheduleDateDialog.value = true;
}

// 保存课表日期
async function handleSaveScheduleDate() {
  if (!editingSchedule.value.first_day_date) {
    ElMessage.warning('请选择日期');
    return;
  }

  try {
    // 解析日期字符串 "YYYY-MM-DD"
    const dateStr = editingSchedule.value.first_day_date;
    const parts = dateStr.split('-');
    const year = parseInt(parts[0]);
    const month = parseInt(parts[1]) - 1; // 月份从0开始
    const day = parseInt(parts[2]);

    // 使用本地时间创建日期对象
    const date = new Date(year, month, day);

    // 获取时间戳（秒）
    const timestamp = Math.floor(date.getTime() / 1000);

    console.log('日期:', dateStr, '时间戳:', timestamp, '日期对象:', date);

    await invoke('update_schedule_info', {
      scheduleId: editingSchedule.value.id,
      firstDay: timestamp,
    });

    // 如果是当前课表，更新配置
    if (editingSchedule.value.id === currentScheduleId.value) {
      config.value.first_day = timestamp;
      // 重新计算当前周次
      const week = Math.floor((Date.now() / 1000 - timestamp) / (7 * 24 * 60 * 60)) + 1;
      currentWeek.value = Math.max(Math.min(week, 20), 1);
    }

    ElMessage.success('保存成功');
    showScheduleDateDialog.value = false;
    await loadScheduleList();
  } catch (e) {
    console.error('保存失败:', e);
    ElMessage.error(`保存失败: ${e}`);
  }
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

    // 计算当前周次（first_day 是秒级时间戳，需要转换为毫秒级）
    if (appConfig.first_day) {
      const week = Math.floor((Date.now() - appConfig.first_day * 1000) / (7 * 24 * 60 * 60 * 1000)) + 1;
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
  setupImportListener(async (result) => {
    // result 包含 { schedule_id, course_count, schedule_name }
    console.log('导入成功:', result);

    loading.value = false;
    isImportingFromBrowser.value = false;

    // 显示成功消息
    ElMessage.success(`导入成功！已导入 ${result.course_count} 门课程：${result.schedule_name}`);

    // 刷新课表列表
    await loadScheduleList();

    // 重新加载当前课表数据
    await loadCachedSchedule();

    // 刷新配置
    await loadConfig();
  });

  // 尝试加载缓存
  const hasCache = await loadCachedSchedule();

  if (hasCache) {
    await loadConfig();
  } else {
    // 如果没有缓存,直接打开浏览器（不显示导入对话框）
    console.log('没有缓存课表，打开浏览器导入');
    isImportingFromBrowser.value = true;
    await openBrowser();
  }

  // 初始化深色模式
  const savedTheme = localStorage.getItem('theme');
  const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  
  if (savedTheme === 'dark' || (!savedTheme && systemDark)) {
    isDark.value = true;
    document.documentElement.classList.add('dark');
    document.documentElement.classList.remove('light');
  } else {
    isDark.value = false;
    document.documentElement.classList.add('light');
    document.documentElement.classList.remove('dark');
  }
});

// 深色模式
const isDark = ref(false);

function toggleTheme() {
  isDark.value = !isDark.value;
  if (isDark.value) {
    document.documentElement.classList.add('dark');
    document.documentElement.classList.remove('light');
    localStorage.setItem('theme', 'dark');
  } else {
    document.documentElement.classList.add('light');
    document.documentElement.classList.remove('dark');
    localStorage.setItem('theme', 'light');
  }
}
</script>

<style>
:root {
    /* 现代极简配色 - 浅色模式 - 实体背景 */
    --primary-color: #6366f1;
    --primary-gradient: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
    --bg-color: #f5f7fa;
    --surface-color-light: #ffffff; /* Dialog Background */
    --surface-color-strong: #ffffff;
    --card-bg: #f8fafc; /* Slightly darker than white for cards */
    --input-bg: #f1f5f9; /* Inputs */
    --surface-blur: none;
    --text-main: #1e293b;
    --text-secondary: #475569;
    --text-tertiary: #94a3b8;
    --border-color: #e2e8f0;
    --border-radius-base: 16px;
    --border-radius-lg: 20px;
    --border-radius-sm: 8px;
    --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
    --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.08), 0 2px 4px -1px rgba(0, 0, 0, 0.04); /* Stronger shadow */
    --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.08), 0 4px 6px -2px rgba(0, 0, 0, 0.04);
    --shadow-active: 0 0 0 2px rgba(99, 102, 241, 0.2);
    
    --font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  }
  
  /* 深色模式 - 支持系统偏好和手动切换 */
  html.dark :root {
    /* 现代极简配色 - 深色模式 - 实体背景 */
    --primary-color: #818cf8;
    --primary-gradient: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
    --bg-color: #0f172a;
    --surface-color-light: #1e293b; /* Slate 800 */
    --surface-color-strong: #1e293b;
    --card-bg: #283446; /* Slightly lighter than 800, distinctive */
    --input-bg: #0f172a; /* Darker for proper contrast */
    --text-main: #f1f5f9;
    --text-secondary: #cbd5e1;
    --text-tertiary: #94a3b8;
    --border-color: #334155;
    --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.4);
    --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.4);
  }
  
  @media (prefers-color-scheme: dark) {
    /* 默认跟随系统，如果未设置手动偏好 */
    :root:not([class*="light"]) {
      --primary-color: #818cf8;
      --primary-gradient: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
      --bg-color: #0f172a;
      --surface-color-light: #1e293b;
      --surface-color-strong: #1e293b;
      --card-bg: #283446;
      --input-bg: #0f172a;
      --text-main: #f1f5f9;
      --text-secondary: #cbd5e1;
      --text-tertiary: #94a3b8;
      --border-color: #334155;
      --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.4);
      --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.4);
    }
  }

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
  -webkit-tap-highlight-color: transparent;
}

body {
  font-family: var(--font-family);
  color: var(--text-main);
  background-color: var(--bg-color);
  -webkit-font-smoothing: antialiased;
}

#app {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background-color: var(--bg-color);
  background-size: cover;
  background-position: center;
  transition: background-color 0.3s ease;
}

/* 主题切换按钮 */
.theme-toggle {
  position: fixed;
  bottom: 24px;
  right: 24px;
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background-color: var(--surface-color-strong);
  backdrop-filter: var(--surface-blur);
  border: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: var(--shadow-lg);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 100;
  color: var(--text-secondary);
}

.theme-toggle:hover {
  transform: translateY(-2px) rotate(15deg);
  background-color: var(--surface-color-strong);
  color: var(--primary-color);
  box-shadow: 0 8px 20px rgba(99, 102, 241, 0.25);
}

.theme-toggle:active {
  transform: scale(0.92);
}

html.dark .theme-toggle {
  color: #ffd700; /* Gold for sun */
}

/* 导航栏 - 浮动极简风格 */
.navbar {
  height: 64px; /* Slightly taller */
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  background-color: transparent; /* Remove solid background */
  z-index: 50;
}

.navbar-left {
  position: absolute;
  left: 24px;
  cursor: pointer;
  width: 40px;
  height: 40px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--surface-color-light);
  backdrop-filter: var(--surface-blur);
  border: 1px solid var(--border-color);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  color: var(--text-main);
}

.navbar-left:hover {
  transform: translateY(-1px);
  background-color: var(--surface-color-strong);
  box-shadow: var(--shadow-md);
}

.navbar-left:active {
  transform: scale(0.95);
}

.navbar-center {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 16px;
  font-weight: 600;
  padding: 8px 16px;
  border-radius: 20px;
  background-color: var(--surface-color-light);
  backdrop-filter: var(--surface-blur);
  border: 1px solid var(--border-color);
  transition: all 0.2s;
  color: var(--text-main);
}

.navbar-center:hover {
  background-color: var(--surface-color-strong);
  box-shadow: var(--shadow-md);
}

/* 加载状态 */
.loading {
  position: fixed;
  bottom: 30px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 24px;
  background-color: var(--surface-color-strong);
  backdrop-filter: var(--surface-blur);
  border-radius: 30px;
  box-shadow: var(--shadow-lg);
  border: 1px solid var(--border-color);
  z-index: 1000;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-main);
}

/* 主内容 */
.main-content {
  height: calc(100vh - 64px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 0 16px 16px; /* Added padding for floating content */
}

/* 星期栏 */
.week-bar {
  display: flex;
  height: 44px;
  margin-bottom: 8px;
  background-color: transparent;
  border: none;
  flex-shrink: 0;
  padding: 0 8px; /* Align with content */
}

.week-bar-month {
  width: 6%;
  min-width: 45px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 700;
  color: var(--text-main);
  opacity: 0.8;
}

.week-bar-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  color: var(--text-secondary);
}

.week-bar-item .date {
  color: var(--text-tertiary);
  font-size: 11px;
  margin-top: 2px;
  font-weight: 500;
}

/* 统一对话框样式 */
.custom-dialog .el-dialog {
  border-radius: 24px !important;
  background-color: var(--bg-color) !important;
  box-shadow: var(--shadow-lg) !important;
  overflow: hidden;
  border: 1px solid var(--border-color) !important;
}

.custom-dialog .el-dialog__header {
  display: none;
}

.custom-dialog .el-dialog__body {
  padding: 0 !important;
  background-color: transparent !important;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px 28px;
  background-color:transparent;
}

.dialog-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-main);
  letter-spacing: -0.5px;
}

.dialog-close {
  cursor: pointer;
  padding: 8px;
  border-radius: 50%;
  transition: all 0.2s;
  color: var(--text-tertiary);
  background-color: rgba(0,0,0,0.03);
}

.dialog-close:hover {
  background-color: rgba(0,0,0,0.08);
  color: var(--text-main);
  transform: rotate(90deg);
}

.dialog-content {
  padding: 0 28px 28px;
  max-height: 60vh;
  overflow-y: auto;
  background-color: transparent;
}

.dialog-footer {
  padding: 20px 28px;
  border-top: 1px solid var(--border-color);
  background-color: var(--surface-color-light);
}

/* 设置表单样式 */
.setting-group {
  margin-bottom: 24px;
}

.setting-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-main);
  margin-bottom: 12px;
  display: block;
}

.setting-hint {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 8px;
}

.custom-input .el-input__wrapper {
  background-color: var(--surface-color-light) !important;
  box-shadow: none !important;
  border: 1px solid transparent !important;
  border-radius: 12px;
  padding: 10px 15px;
  transition: all 0.2s;
}

.custom-input .el-input__wrapper:hover {
  background-color: var(--surface-color-strong) !important;
}

.custom-input .el-input__wrapper.is-focus {
  background-color: #fff !important;
  box-shadow: 0 0 0 2px var(--primary-color) !important;
}

.custom-input .el-input__inner {
  color: var(--text-main) !important;
  height: auto;
}

/* 输入框深色模式适配 */
html.dark .custom-input .el-input__wrapper.is-focus {
  background-color: rgba(255,255,255,0.1) !important;
}

/* 数字输入框适配 */
.custom-number .el-input-number__decrease,
.custom-number .el-input-number__increase {
  background-color: var(--surface-color-light) !important;
  border: none !important;
  color: var(--text-secondary) !important;
}

.custom-number .el-input__wrapper {
  background-color: var(--surface-color-light) !important;
  box-shadow: none !important;
  border-radius: 12px;
}

/* 时间列表 */
.time-list {
  background-color: var(--surface-color-light);
  border-radius: 16px;
  padding: 16px;
  border: 1px solid var(--border-color);
}

.time-item {
  border-bottom: 1px solid var(--border-color);
  padding: 10px 0;
}

/* 按钮通用样式 */
.custom-button {
  border-radius: 12px;
  padding: 12px 24px;
  font-weight: 600;
  transition: all 0.2s;
  height: auto;
}

.primary-button {
  background: var(--primary-gradient);
  border: none;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
}

.primary-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(99, 102, 241, 0.4);
}

.primary-button:active {
  transform: scale(0.98);
}

/* 列表卡片样式 (Appearance & Schedule) */
.appearance-cards {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.appearance-card {
  padding: 24px;
  background-color: var(--surface-color-light);
  border-radius: 20px;
  cursor: pointer;
  transition: all 0.2s;
  text-align: center;
  border: 1px solid var(--border-color);
}

.appearance-card:hover {
  background-color: var(--surface-color-strong);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
  border-color: var(--primary-color);
}

.schedule-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.schedule-card {
  padding: 20px;
  background-color: var(--surface-color-light);
  border-radius: 20px;
  border: 1px solid var(--border-color);
  transition: all 0.2s;
}

.schedule-card:hover {
  background-color: var(--surface-color-strong);
  transform: translateY(-1px);
  box-shadow: var(--shadow-sm);
}

.schedule-card.active {
  background-color: var(--surface-color-strong);
  border-color: var(--primary-color);
  box-shadow: 0 0 0 1px var(--primary-color);
}

.schedule-name {
  color: var(--text-main);
  font-weight: 600;
  font-size: 16px;
}

.meta-item {
  color: var(--text-secondary);
}

/* Empty State */
.empty-text { color: var(--text-secondary); }
.empty-hint { color: var(--text-tertiary); }



/* --- Unified Modern Dialog Styles (v3.1 - Fixed Selectors) --- */

/* The .custom-dialog CLASS is applied to the .el-dialog ELEMENT itself */
.custom-dialog {
  background-color: transparent !important;
  box-shadow: none !important;
  margin: 0 !important; /* Managed by align-center */
  width: 90vw !important; /* Forced visual width relative to viewport */
  max-width: 480px;
  min-width: 300px !important;
  max-height: 90vh !important;
  display: flex !important;
  flex-direction: column !important;
  position: relative !important;
  transform: none !important;
  /* Reset positioning overrides as align-center handles it, but ensure no weird offsets */
  left: auto !important;
  top: auto !important;
  --el-dialog-bg-color: transparent !important;
  border-radius: var(--border-radius-lg) !important;
}

/* Ensure no child forces width beyond 100% */
.custom-dialog .el-dialog__body {
  padding: 0 !important;
  background-color: var(--surface-color-strong) !important;
  border-radius: var(--border-radius-lg);
  box-shadow: var(--shadow-lg);
  border: 1px solid var(--border-color);
  color: var(--text-main);
  display: flex !important;
  flex-direction: column !important;
  flex: 1 !important;
  overflow: hidden !important;
  opacity: 1 !important;
  height: auto !important;
  width: 100% !important; /* Ensure body fits dialog */
  box-sizing: border-box;
}

/* Header - Fixed */
.dialog-header {
  padding: 16px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--border-color);
  background-color: var(--surface-color-light); /* Distinct header bg */
  flex-shrink: 0;
  z-index: 10;
}

.dialog-title {
  font-size: 17px;
  font-weight: 700;
  color: var(--text-main);
}

.dialog-close-btn {
  width: 32px;
  height: 32px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--text-tertiary);
  transition: all 0.2s;
}

.dialog-close-btn:hover {
  background-color: rgba(0,0,0,0.05);
  color: var(--text-main);
}

/* Content - Scrollable */
.dialog-content {
  padding: 20px;
  overflow-y: auto !important; /* Enable scrolling */
  overflow-x: hidden;
  flex: 1; /* Take remaining space */
  -webkit-overflow-scrolling: touch;
}

/* Footer - Fixed */
.dialog-footer {
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  background-color: var(--surface-color-light);
  flex-shrink: 0;
  z-index: 10;
}

/* Scrollbar Styling */
.dialog-content::-webkit-scrollbar {
  width: 6px;
}
.dialog-content::-webkit-scrollbar-track {
  background: transparent;
}
.dialog-content::-webkit-scrollbar-thumb {
  background-color: var(--border-color);
  border-radius: 3px;
}
.dialog-content::-webkit-scrollbar-thumb:hover {
  background-color: var(--text-tertiary);
}

.no-scrollbar::-webkit-scrollbar {
  display: none;
}
.no-scrollbar {
  scrollbar-width: none;
}

/* --- Appearance Grid --- */
.appearance-grid {
  display: grid;
  grid-template-columns: 1fr; /* Responsive: start with 1 col on very small screens */
  gap: 16px;
}

/* Tablet+ override */
@media (min-width: 400px) {
  .appearance-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}


.appearance-item {
  background-color: var(--card-bg);
  border-radius: 16px;
  overflow: hidden;
  cursor: pointer;
  border: 1px solid var(--border-color);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: var(--shadow-sm);
}

.appearance-item:hover {
  transform: translateY(-3px);
  border-color: var(--primary-color);
  box-shadow: var(--shadow-md);
}

.item-preview {
  height: 100px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(99, 102, 241, 0.05);
  position: relative;
  overflow: hidden;
  color: var(--primary-color);
}

.item-preview.delete-preview {
  background-color: rgba(239, 68, 68, 0.05);
  color: #ef4444;
}

.hover-overlay {
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  background-color: rgba(0,0,0,0.4);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: white;
  opacity: 0;
  transition: opacity 0.2s;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
  backdrop-filter: blur(2px);
}

.appearance-item:hover .hover-overlay {
  opacity: 1;
}

.item-info {
  padding: 12px;
}

.item-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-main);
  margin-bottom: 4px;
}

.item-desc {
  font-size: 11px;
  color: var(--text-tertiary);
}

/* --- Schedule List (Modern) --- */
.schedule-list-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 200px;
}

.schedule-list-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  background-color: var(--card-bg);
  border-radius: 14px;
  border: 1px solid var(--border-color);
  transition: all 0.2s;
  cursor: pointer;
  box-shadow: var(--shadow-sm);
}

.schedule-list-item:hover {
  background-color: var(--surface-color-strong);
  transform: translateX(4px);
  box-shadow: var(--shadow-sm);
}

.schedule-list-item.is-active {
  background: var(--surface-color-strong);
  border-color: var(--primary-color);
  box-shadow: var(--shadow-active);
}

.item-main {
  flex: 1;
  min-width: 0; /* Crucial for flex content truncation */
  margin-right: 12px;
}

.item-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.item-name {
  font-weight: 700;
  color: var(--text-main);
  font-size: 15px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.status-tag {
  flex-shrink: 0; /* Tag shouldn't shrink */
}

.item-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: var(--text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-actions {
  display: flex;
  gap: 4px; /* Tighter gap */
  opacity: 1; /* Always visible for mobile friendliness or handle via hover on desktop */
  flex-shrink: 0;
}

/* --- Detail Sheet Styles --- */
.detail-sheet-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.4);
  z-index: 1000;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
}

.detail-sheet {
  background-color: var(--card-bg); /* Use card bg for theme adaptation */
  border-top-left-radius: 20px;
  border-top-right-radius: 20px;
  padding: 10px 24px 40px 24px;
  box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.15);
  position: relative;
  border-top: 1px solid rgba(255,255,255,0.1);
}

.detail-handle {
  width: 40px;
  height: 4px;
  background-color: rgba(100, 116, 139, 0.2);
  border-radius: 2px;
  margin: 10px auto 20px auto;
}

.detail-header {
  margin-bottom: 24px;
}

.detail-name {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-main);
  line-height: 1.4;
}

.detail-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.detail-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.detail-icon-box {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  flex-shrink: 0;
}

/* Icon Box Colors */
.detail-icon-box.blue { background-color: rgba(59, 130, 246, 0.1); color: #3b82f6; }
.detail-icon-box.green { background-color: rgba(16, 185, 129, 0.1); color: #10b981; }
.detail-icon-box.orange { background-color: rgba(249, 115, 22, 0.1); color: #f97316; }
.detail-icon-box.purple { background-color: rgba(139, 92, 246, 0.1); color: #8b5cf6; }

/* Dark mode adjustments for icon boxes handled by opacity mostly, but let's ensure visibility */
html.dark .detail-icon-box.blue { background-color: rgba(59, 130, 246, 0.2); }
html.dark .detail-icon-box.green { background-color: rgba(16, 185, 129, 0.2); }
html.dark .detail-icon-box.orange { background-color: rgba(249, 115, 22, 0.2); }
html.dark .detail-icon-box.purple { background-color: rgba(139, 92, 246, 0.2); }


.detail-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.detail-label {
  font-size: 11px;
  color: var(--text-tertiary);
  font-weight: 500;
}

.detail-value {
  font-size: 15px;
  color: var(--text-main);
  font-weight: 600;
}

/* Animations */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: opacity 0.3s ease;
}

.slide-up-enter-active .detail-sheet,
.slide-up-leave-active .detail-sheet {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
}

.slide-up-enter-from .detail-sheet,
.slide-up-leave-to .detail-sheet {
  transform: translateY(100%);
}



/* On desktop, we can hide actions until hover if preferred, but for now let's keep them accessible or check previous logic */
@media (hover: hover) {
  .item-actions {
    opacity: 0;
    transform: translateX(10px);
    transition: all 0.2s;
  }
  .schedule-list-item:hover .item-actions {
    opacity: 1;
    transform: translateX(0);
  }
}

.action-btn {
  width: 32px;
  height: 32px;
  font-size: 16px;
  color: var(--text-secondary);
  border-radius: 8px; /* Softer shape */
}

.action-btn:hover {
  background-color: rgba(99, 102, 241, 0.1);
  color: var(--primary-color);
}

.action-btn.danger:hover {
  background-color: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 0;
  color: var(--text-tertiary);
  gap: 12px;
}

/* Modern Input */
.modern-input .el-input__wrapper,
.modern-number-input .el-input__wrapper {
  background-color: var(--input-bg) !important;
  box-shadow: none !important;
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 4px 12px;
  transition: all 0.2s;
}

.modern-input .el-input__wrapper:hover,
.modern-input .el-input__wrapper.is-focus {
  background-color: var(--input-bg) !important;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.1) !important;
}

/* Appearance Items - Cards */
.appearance-item {
  background-color: var(--card-bg);
  border-radius: 16px;
  overflow: hidden;
  cursor: pointer;
  border: 1px solid var(--border-color); /* Stronger border */
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: var(--shadow-sm); /* Add shadow by default */
}

/* Schedule List Items */
.schedule-list-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  background-color: var(--card-bg);
  border-radius: 14px;
  border: 1px solid var(--border-color); /* Stronger border */
  transition: all 0.2s;
  cursor: pointer;
  box-shadow: var(--shadow-sm); /* Add shadow */
  margin-bottom: 8px;
}

.schedule-list-item:hover {
  transform: translateY(-2px); /* Slight lift */
  background-color: var(--card-bg); /* Keep base color */
  border-color: var(--primary-color); /* Highlight border */
  box-shadow: var(--shadow-md);
}

.schedule-list-item.is-active {
  background: var(--card-bg);
  border-color: var(--primary-color);
  box-shadow: 0 0 0 1px var(--primary-color), var(--shadow-sm);
}

/* Time Inputs */
.time-mini-input .el-input__wrapper {
  background-color: var(--input-bg) !important;
}

/* Dark Mode cleanup - most handled by vars now, just ensuring specifics */
html.dark .modern-input .el-input__wrapper {
  background-color: var(--input-bg) !important;
}

html.dark .appearance-item,
html.dark .schedule-list-item {
  background-color: var(--card-bg);
}

html.dark .schedule-list-item:hover {
  background-color: var(--card-bg); /* Avoid color shift, rely on border/shadow */
  border-color: var(--primary-color);
}
</style>
