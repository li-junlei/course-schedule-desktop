<template>
  <div 
    class="schedule-container"
    ref="containerRef"
    @mousedown="startSwipe"
    @mousemove="handleSwipe"
    @mouseup="endSwipe"
    @mouseleave="endSwipe"
    @touchstart="startSwipeTouch"
    @touchmove="handleSwipeTouch"
    @touchend="endSwipe"
  >
    <!-- 左侧时间栏 -->
    <div class="time-bar">
      <div v-for="(time, index) in courseTime" :key="index" class="time-slot">
        <span class="period-number">{{ index + 1 }}</span>
        <span class="period-time">{{ time }}</span>
      </div>
    </div>

    <!-- 右侧课表区域 -->
    <div 
      class="schedule-area" 
      :style="{ 
        backgroundImage: bgImage ? `url(${bgImage})` : 'none',
        transform: `translateX(${swipeOffset}px)`,
        transition: isSwipingEnd ? 'transform 0.3s ease' : 'none'
      }"
    >
      <!-- 假期提示 -->
      <div v-if="week > endWeek && courses.length > 0" class="vacation">
        <span>放假了，出去玩吧！</span>
      </div>

      <!-- 课程卡片 -->
      <CourseCard
        v-for="(course, index) in courses"
        :key="index"
        :course="course"
        :week="week"
        :color="colors[index % colors.length]"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import CourseCard from './CourseCard.vue';
import type { Course, PeriodTime } from '../types';

interface Props {
  courses: Course[];
  week: number;
  endWeek: number;
  colors: string[];
  bgImage?: string;
  maxPeriods?: number;
  periodTimes?: PeriodTime[]; // Custom time labels with start and end
}

const props = withDefaults(defineProps<Props>(), {
  bgImage: '',
  maxPeriods: 13,
  periodTimes: () => [],
});

const emit = defineEmits<{
  (e: 'update:week', value: number): void;
}>();

// Dynamic Time Slots
const courseTime = computed(() => {
  // If custom times provided and length matches (or close enough), use them.
  // We prioritize maxPeriods length.
  const slots = [];
  for (let i = 0; i < props.maxPeriods; i++) {
      if (props.periodTimes && props.periodTimes[i]) {
          const time = props.periodTimes[i];
          // Format as "start-end"
          slots.push(`${time.start}-${time.end}`);
      } else {
          // Default generation (continuous periods, no breaks)
          const p = i + 1;
          // Start from 8:00, each period is 45 minutes with 10 min break
          const startHour = 8 + Math.floor((p - 1) * 55 / 60);
          const startMin = ((p - 1) * 55) % 60;

          const endMin = (startMin + 45) % 60;
          const endHour = startHour + Math.floor((startMin + 45) / 60);

          slots.push(`${startHour}:${startMin.toString().padStart(2, '0')}-${endHour}:${endMin.toString().padStart(2, '0')}`);
      }
  }
  return slots;
});

// CSS Variable for Row Height
const rowHeightStr = computed(() => `${100 / props.maxPeriods}%`);

// 滑动状态
const containerRef = ref<HTMLElement | null>(null);
const isSwiping = ref(false);
const isSwipingEnd = ref(false);
const swipeStartX = ref(0);
const swipeOffset = ref(0);
const swipeThreshold = 80; // 滑动阈值

function startSwipe(e: MouseEvent) {
  isSwiping.value = true;
  isSwipingEnd.value = false;
  swipeStartX.value = e.clientX;
  swipeOffset.value = 0;
}

function startSwipeTouch(e: TouchEvent) {
  isSwiping.value = true;
  isSwipingEnd.value = false;
  swipeStartX.value = e.touches[0].clientX;
  swipeOffset.value = 0;
}

function handleSwipe(e: MouseEvent) {
  if (!isSwiping.value) return;
  const diff = e.clientX - swipeStartX.value;
  // 限制滑动范围
  swipeOffset.value = Math.max(-150, Math.min(150, diff));
}

function handleSwipeTouch(e: TouchEvent) {
  if (!isSwiping.value) return;
  const diff = e.touches[0].clientX - swipeStartX.value;
  swipeOffset.value = Math.max(-150, Math.min(150, diff));
}

function endSwipe() {
  if (!isSwiping.value) return;
  
  isSwiping.value = false;
  isSwipingEnd.value = true;

  // 判断滑动方向和距离
  if (swipeOffset.value > swipeThreshold) {
    // 向右滑动 - 上一周
    const newWeek = props.week > 1 ? props.week - 1 : 20;
    emit('update:week', newWeek);
  } else if (swipeOffset.value < -swipeThreshold) {
    // 向左滑动 - 下一周
    const newWeek = props.week < 20 ? props.week + 1 : 1;
    emit('update:week', newWeek);
  }

  // 复位
  swipeOffset.value = 0;
  
  setTimeout(() => {
    isSwipingEnd.value = false;
  }, 300);
}
</script>

<style scoped>
.schedule-container {
  display: flex;
  width: 100%;
  height: 100%;
  user-select: none;
  cursor: grab;
}

.schedule-container:active {
  cursor: grabbing;
}

/* 左侧时间栏 */
.time-bar {
  width: 6%;
  min-width: 45px;
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: rgba(255, 255, 255, 0.8);
}

.time-slot {
  height: v-bind(rowHeightStr);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  border-bottom: 1px dashed rgba(0,0,0,0.05); /* Optional: add separator */
}

.period-number {
  font-size: 13px;
  font-weight: bold;
}

.period-time {
  color: #999;
  font-size: 9px;
  white-space: nowrap;
}

/* 右侧课表区域 */
.schedule-area {
  position: relative;
  flex: 1;
  width: 94%;
  height: 100%;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
}

/* 假期提示 */
.vacation {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 28px;
  color: #76c26b;
  text-align: center;
  width: 100%;
  font-weight: bold;
}

/* 深色模式 */
@media (prefers-color-scheme: dark) {
  .time-bar {
    background-color: rgba(30, 30, 30, 0.8);
  }

  .period-number {
    color: #fff;
  }
}
</style>
