<template>
  <Transition name="popup">
    <div v-if="show" class="popup-overlay" @click.self="$emit('close')">
      <div class="popup-content">
        <div class="popup-title">宿院速办</div>
        <div class="popup-menu">
          <div class="menu-item" @click="$emit('schedule-manage')">
            <el-icon :size="28"><Collection /></el-icon>
            <span>课表管理</span>
          </div>
          <div class="menu-item" @click="$emit('upload-bg')">
            <el-icon :size="28"><Picture /></el-icon>
            <span>自定义背景</span>
          </div>
          <div class="menu-item" @click="$emit('delete-bg')">
            <el-icon :size="28"><Close /></el-icon>
            <span>删除背景</span>
          </div>
          <div class="menu-item" @click="$emit('settings')">
            <el-icon :size="28"><Setting /></el-icon>
            <span>设置</span>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { Collection, Picture, Close, Setting } from '@element-plus/icons-vue';

defineProps<{
  show: boolean;
}>();

defineEmits<{
  (e: 'close'): void;
  (e: 'schedule-manage'): void;
  (e: 'upload-bg'): void;
  (e: 'delete-bg'): void;
  (e: 'settings'): void;
}>();
</script>

<style scoped>
.popup-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.3);
  z-index: 1000;
}

.popup-content {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  background-color: white;
  border-radius: 0 0 16px 16px;
  padding: 20px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.popup-title {
  text-align: center;
  font-size: 18px;
  font-weight: bold;
  margin-bottom: 20px;
  padding-top: 10px;
  letter-spacing: 2px;
}

.popup-menu {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
  padding: 10px 0;
}

.menu-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  cursor: pointer;
  border-radius: 8px;
  transition: background-color 0.2s;
  font-size: 14px;
}

.menu-item:hover {
  background-color: #f5f5f5;
}

.menu-item .el-icon {
  color: #333;
}

/* 动画 */
.popup-enter-active,
.popup-leave-active {
  transition: opacity 0.3s ease;
}

.popup-enter-active .popup-content,
.popup-leave-active .popup-content {
  transition: transform 0.3s ease;
}

.popup-enter-from,
.popup-leave-to {
  opacity: 0;
}

.popup-enter-from .popup-content,
.popup-leave-to .popup-content {
  transform: translateY(-100%);
}

/* 深色模式 */
@media (prefers-color-scheme: dark) {
  .popup-content {
    background-color: #1a1a1a;
    color: #fff;
  }

  .menu-item:hover {
    background-color: #333;
  }

  .menu-item .el-icon {
    color: #fff;
  }
}
</style>
