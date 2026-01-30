<template>
  <div
    v-if="isVisible"
    class="course-card"
    :style="{
        top: topPosition,
        left: leftPosition,
        height: cardHeight,
    }"
    @click="showDetails"
  >
    <div 
      class="card-inner"
      :style="{ 
        backgroundColor: color,
        alignItems: isShortCard ? 'flex-start' : 'center'
      }"
    >
      <span class="course-text">{{ course.name }}@{{ course.location }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { ElMessage } from 'element-plus';
import type { Course } from '../types';

interface Props {
  course: Course;
  week: number;
  color: string;
  maxPeriods?: number;
}

const props = withDefaults(defineProps<Props>(), {
  maxPeriods: 13,
});

/**
 * 检查课程是否在指定周次显示
 */
const isVisible = computed(() => {
  const { course, week } = props;

  // 检查是否在周次范围内 (weeks现在是完整的周次列表)
  const inRange = course.weeks.includes(week);

  // 检查单双周 (作为备用逻辑)
  const weekMatch = course.week_type === 0 || (course.week_type == 1 && week % 2 != 0) || (course.week_type == 2 && week % 2 == 0);

  return inRange && weekMatch;
});

/**
 * 判断是否是短卡片（单节课）
 */
const isShortCard = computed(() => {
  const { periods } = props.course;
  return isNaN(periods[0]) || !periods[1] || periods[0] === 5;
});

/**
 * 计算顶部位置
 */
const topPosition = computed(() => {
  const { periods } = props.course;
  const firstPeriod = periods[0];

  if (isNaN(firstPeriod)) {
    return '0%';
  }

  // 简单线性布局: (节次 - 1) * (100 / maxPeriods)
  // 如果需要特殊的“中午”休息区，需要更复杂的逻辑或配置
  // 目前根据用户反馈，主要是因为12-13节无法显示，所以优先支持扩展的线性列表
  const unitHeight = 100 / props.maxPeriods;
  return `${(firstPeriod - 1) * unitHeight}%`;
});

/**
 * 计算左侧位置
 */
const leftPosition = computed(() => {
  const dayOfWeek = props.course.day_of_week;
  return `${((dayOfWeek - 1) * 94) / 7}%`;
});

/**
 * 计算高度
 */
const cardHeight = computed(() => {
  const { periods } = props.course;
  const firstPeriod = periods[0];
  const lastPeriod = periods[periods.length - 1];

  if (isNaN(firstPeriod) || !lastPeriod) {
    const unitHeight = 100 / props.maxPeriods;
    return `${unitHeight}%`;
  }

  const unitHeight = 100 / props.maxPeriods;
  return `${(lastPeriod - firstPeriod + 1) * unitHeight}%`;
});

/**
 * 显示课程详情
 */
function showDetails() {
  ElMessage({
    message: `教师：${props.course.teacher}\n地点：${props.course.location}`,
    type: 'info',
    duration: 2800,
    showClose: true,
  });
}
</script>

<style scoped>
.course-card {
  position: absolute;
  width: 14%;
  padding: 2px 4px;
  box-sizing: border-box;
  overflow: hidden;
  cursor: pointer;
}

.card-inner {
  width: 100%;
  height: 100%;
  border-radius: 8px; /* Slightly tighter radius for mobile look */
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  box-sizing: border-box;
  transition: transform 0.2s, box-shadow 0.2s;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1); 
  opacity: 0.92; /* Transparency */
  backdrop-filter: blur(2px);
}

.card-inner:hover {
  transform: scale(1.03);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
  z-index: 10;
}

.course-text {
  display: block;
  color: white;
  font-size: 11px;
  font-weight: bold;
  word-break: break-word;
  text-align: center;
  line-height: 1.3;
  margin: 2px;
}
</style>
