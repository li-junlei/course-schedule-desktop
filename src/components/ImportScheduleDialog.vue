<template>
  <el-dialog
    v-model="dialogVisible"
    title=""
    width="90%"
    align-center
    :show-close="false"
    class="custom-dialog import-dialog"
    append-to-body
    style="max-width: 480px;"
    @close="handleClose"
  >
    <div class="dialog-header">
      <div class="dialog-title">导入课表</div>
      <div class="dialog-close-btn" @click="handleClose">
        <el-icon :size="20"><Close /></el-icon>
      </div>
    </div>

    <div class="dialog-content">
      <!-- 教务系统选择 -->
      <div class="setting-item">
        <div class="setting-label">选择教务系统</div>
        <el-select
          v-model="selectedSystemId"
          placeholder="请选择教务系统"
          class="modern-input"
          style="width: 100%"
          :disabled="eduSystems.length === 0"
        >
          <el-option
            v-for="system in enabledSystems"
            :key="system.id"
            :label="system.name"
            :value="system.id"
          >
            <div class="system-option">
              <span class="system-name">{{ system.name }}</span>
              <span v-if="system.id === lastSelectedId" class="system-badge">上次使用</span>
            </div>
          </el-option>
        </el-select>
        <div v-if="eduSystems.length === 0" class="setting-desc warning">
          暂无可用的教务系统配置
        </div>
        <div v-else class="setting-desc">
          将跳转到所选教务系统的登录页面
        </div>
      </div>

      <!-- 课表名称输入 -->
      <div class="setting-item">
        <div class="setting-label">课表名称</div>
        <el-input
          v-model="scheduleName"
          placeholder="请输入课表名称"
          class="modern-input"
          clearable
          @keyup.enter="handleConfirm"
        >
          <template #prefix>
            <el-icon><Collection /></el-icon>
          </template>
        </el-input>
        <div class="setting-desc">为导入的课表设置一个易于识别的名称</div>
      </div>
    </div>

    <div class="dialog-footer">
      <el-button @click="handleClose" class="modern-button">取消</el-button>
      <el-button
        type="primary"
        @click="handleConfirm"
        class="modern-button primary"
        :disabled="!selectedSystemId || !scheduleName.trim()"
      >
        确定
      </el-button>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Close, Collection } from '@element-plus/icons-vue';
import type { EduSystem } from '../types';

interface Props {
  modelValue: boolean;
  eduSystems: EduSystem[];
  lastSelectedId?: string;
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void;
  (e: 'confirm', data: { systemId: string; scheduleName: string }): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const dialogVisible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
});

const selectedSystemId = ref('');
const scheduleName = ref('');

// 过滤出启用的教务系统
const enabledSystems = computed(() => {
  return props.eduSystems.filter(s => s.enabled);
});

// 监听 dialog 打开，初始化默认值
watch(() => props.modelValue, (newVal) => {
  if (newVal) {
    // Dialog 打开时，设置默认值
    selectedSystemId.value = props.lastSelectedId || (enabledSystems.value[0]?.id || '');
    const now = new Date();
    const dateStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
    scheduleName.value = `从浏览器导入 ${dateStr}`;
  }
});

const handleClose = () => {
  dialogVisible.value = false;
};

const handleConfirm = () => {
  if (!selectedSystemId.value) {
    return;
  }
  if (!scheduleName.value.trim()) {
    return;
  }

  emit('confirm', {
    systemId: selectedSystemId.value,
    scheduleName: scheduleName.value.trim()
  });

  dialogVisible.value = false;
};
</script>

<style scoped>
.import-dialog .dialog-content {
  padding: 20px 28px;
}

.setting-item {
  margin-bottom: 24px;
}

.setting-item:last-child {
  margin-bottom: 0;
}

.setting-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-main);
  margin-bottom: 10px;
  display: block;
}

.setting-desc {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 8px;
  line-height: 1.4;
}

.setting-desc.warning {
  color: #f56c6c;
}

.system-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.system-name {
  flex: 1;
}

.system-badge {
  font-size: 11px;
  padding: 2px 6px;
  background-color: rgba(103, 194, 58, 0.1);
  color: #67c23a;
  border-radius: 4px;
  margin-left: 8px;
}

.modern-input .el-input__wrapper,
.modern-input .el-select__wrapper {
  background-color: var(--input-bg) !important;
  box-shadow: none !important;
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 4px 12px;
  transition: all 0.2s;
}

.modern-input .el-input__wrapper:hover,
.modern-input .el-input__wrapper.is-focus,
.modern-input .el-select__wrapper:hover,
.modern-input .el-select__wrapper.is-focus {
  background-color: var(--input-bg) !important;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.1) !important;
}

.modern-button {
  border-radius: 12px;
  padding: 12px 24px;
  font-weight: 600;
  transition: all 0.2s;
  height: auto;
}

.modern-button.primary {
  background: var(--primary-gradient);
  border: none;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
  color: white;
}

.modern-button.primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(99, 102, 241, 0.4);
}

.modern-button.primary:active {
  transform: scale(0.98);
}

/* 深色模式适配 */
html.dark .modern-input .el-input__wrapper.is-focus,
html.dark .modern-input .el-select__wrapper.is-focus {
  background-color: var(--input-bg) !important;
}
</style>
