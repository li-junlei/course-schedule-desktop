<template>
  <Transition name="week-selector">
    <div v-show="show" class="week-selector">
      <div class="week-list" ref="weekListRef">
        <div
          v-for="w in 20"
          :key="w"
          :id="`week-${w}`"
          class="week-item"
          :class="{ active: w === week }"
          @click="selectWeek(w)"
        >
          <div class="week-title">第{{ w }}周</div>
          <div class="week-dots">
            <div
              v-for="(_, index) in 35"
              :key="index"
              class="dot"
              :style="{ backgroundColor: getDotColor(w, index) }"
            ></div>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import type { Course } from '../types';

interface Props {
  show: boolean;
  week: number;
  courses: Course[];
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:week', value: number): void;
}>();

const weekListRef = ref<HTMLElement | null>(null);

/**
 * 计算每个周次的点阵状态
 */
const weekDotStatus = computed(() => {
  const dots: (DotStatus | null)[] = new Array(35).fill(null);

  for (const course of props.courses) {
    if (isNaN(course.periods[0]) || course.periods[0] === 5) {
      continue;
    }

    // 计算点阵索引
    const periodIndex = Math.floor((course.periods[0] - 1) / 2); // 0-4
    const dayIndex = course.day_of_week - 1; // 0-6
    const dotIndex = periodIndex * 7 + dayIndex; // 0-34

    if (dotIndex >= 0 && dotIndex < 35) {
      if (dots[dotIndex] === null || course.periods.length > dots[dotIndex]!.periodCount) {
        dots[dotIndex] = {
          weeks: course.weeks,
          periodCount: course.periods.length,
          weekType: course.week_type,
        };
      }
    }
  }

  return dots;
});

interface DotStatus {
  weeks: number[];
  periodCount: number;
  weekType: number;
}

/**
 * 获取点的颜色
 */
function getDotColor(week: number, dotIndex: number): string {
  const dotStatus = weekDotStatus.value[dotIndex];
  if (!dotStatus) {
    return '#d4d4d4'; // 灰色 - 无课
  }

  const { weeks, periodCount } = dotStatus;

  // 检查是否在周次范围内 (weeks现在是完整的周次列表)
  const inRange = weeks.includes(week);

  // 检查单双周 (虽然weeks列表已经是准确的，但保留weekType逻辑作为显示区分或其他用途)
  // const weekMatch = weekType === 0 || weekType % 2 === (week % 2);

  if (!inRange) {
    return '#d4d4d4'; // 灰色 - 无课
  }

  // 绿色 - 有课 2 节，蓝色 - 有课 1 节
  return periodCount === 2 ? 'rgb(56,201,153)' : '#76B1E4';
}

/**
 * 选择周次
 */
function selectWeek(w: number) {
  emit('update:week', w);
}

/**
 * 滚动到当前周次
 */
watch(
  () => props.show,
  async (newVal) => {
    if (newVal) {
      await nextTick();
      const el = document.getElementById(`week-${props.week}`);
      if (el && weekListRef.value) {
        el.scrollIntoView({ behavior: 'smooth', inline: 'center' });
      }
    }
  }
);
</script>

<style scoped>
.week-selector {
  width: 100%;
  background-color: rgba(255, 255, 255, 0.95);
  overflow: hidden;
}

.week-list {
  display: flex;
  overflow-x: auto;
  gap: 5px;
  padding: 8px 10px;
  scrollbar-width: thin;
}

.week-list::-webkit-scrollbar {
  height: 4px;
}

.week-list::-webkit-scrollbar-thumb {
  background-color: #ccc;
  border-radius: 2px;
}

.week-item {
  flex-shrink: 0;
  width: 90px;
  padding: 6px 4px;
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  cursor: pointer;
  transition: background-color 0.2s;
}

.week-item:hover {
  background-color: rgba(200, 200, 200, 0.3);
}

.week-item.active {
  background-color: rgb(188, 187, 193);
}

.week-title {
  font-size: 12px;
  margin-bottom: 4px;
  white-space: nowrap;
}

.week-dots {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  grid-template-rows: repeat(5, 1fr);
  gap: 2px;
  width: 70px;
  height: 50px;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

/* 动画 */
.week-selector-enter-active,
.week-selector-leave-active {
  transition: max-height 0.3s ease, opacity 0.3s ease;
}

.week-selector-enter-from,
.week-selector-leave-to {
  max-height: 0;
  opacity: 0;
}

.week-selector-enter-to,
.week-selector-leave-from {
  max-height: 100px;
  opacity: 1;
}

/* 深色模式 */
@media (prefers-color-scheme: dark) {
  .week-selector {
    background-color: rgba(30, 30, 30, 0.95);
  }

  .week-item.active {
    background-color: rgb(80, 80, 85);
  }
}
</style>
