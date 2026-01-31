<template>
  <div class="time-table-manager">
    <!-- Dictionary/List View -->
    <div v-if="!editingId" class="list-view">
      <div class="header-actions">
        <el-button type="primary" class="modern-button primary" @click="startCreate">
          <el-icon><Plus /></el-icon> 新增时间表
        </el-button>
      </div>
      
      <div v-if="loading" class="loading-state">
        <el-icon class="is-loading"><Loading /></el-icon> 加载中...
      </div>
      
      <div v-else class="table-list no-scrollbar">
        <div 
          v-for="table in timeTables" 
          :key="table.id" 
          class="table-item"
          :class="{ 'is-selected': modelValue === table.id }"
          @click="handleSelect(table.id)"
        >
          <div class="item-content">
            <div class="item-name">{{ table.name }}</div>
            <div class="item-desc">包含 {{ table.periods.length }} 节课</div>
          </div>
          <div class="item-actions">
            <el-button 
              circle 
              text 
              @click.stop="startEdit(table)"
              class="action-btn"
            >
              <el-icon><Edit /></el-icon>
            </el-button>
            <el-button 
              v-if="table.id !== 'default'"
              circle 
              text 
              type="danger" 
              @click.stop="handleDelete(table.id)"
              class="action-btn danger"
            >
              <el-icon><Delete /></el-icon>
            </el-button>
          </div>
        </div>
        
        <div v-if="timeTables.length === 0" class="empty-state">
          暂无时间表
        </div>
      </div>
    </div>

    <!-- Edit View -->
    <div v-else class="edit-view">
      <div class="edit-header">
        <el-button link @click="cancelEdit">
          <el-icon><ArrowLeft /></el-icon> 返回列表
        </el-button>
        <div class="edit-title">{{ isCreating ? '新建时间表' : '编辑时间表' }}</div>
        <el-button type="primary" link @click="saveEdit">保存</el-button>
      </div>
      
      <div class="edit-form no-scrollbar">
        <div class="form-item">
          <div class="label">名称</div>
          <el-input v-model="editForm.name" placeholder="请输入时间表名称" class="modern-input" />
        </div>
        
        <div class="form-item">
            <div class="label-row">
                <div class="label">节次设置</div>
                <el-button link type="primary" size="small" @click="addPeriod">
                    <el-icon><Plus /></el-icon> 添加节次
                </el-button>
            </div>
            
            <div class="periods-list">
                <div v-for="(p, index) in editForm.periods" :key="index" class="period-row">
                    <div class="period-index">{{ index + 1 }}</div>
                    <el-input v-model="p.start" placeholder="08:00" class="time-input" />
                    <span class="sep">-</span>
                    <el-input v-model="p.end" placeholder="08:45" class="time-input" />
                    <el-button circle text type="danger" size="small" @click="removePeriod(index)" class="delete-period-btn">
                        <el-icon><Close /></el-icon>
                    </el-button>
                </div>
            </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus, Edit, Delete, Loading, ArrowLeft, Close } from '@element-plus/icons-vue';
import { useTimeTable } from '../composables/useTimeTable';
import type { TimeTable } from '../types';

const props = defineProps<{
  modelValue?: string; // Currently selected ID
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', id: string): void;
  (e: 'select', table: TimeTable): void;
}>();

const { timeTables, loading, listTimeTables, saveTimeTable, deleteTimeTable } = useTimeTable();

// State
const editingId = ref<string | null>(null);
const isCreating = ref(false);
const editForm = ref<TimeTable>({
    id: '',
    name: '',
    periods: []
});

onMounted(() => {
    listTimeTables();
});

function handleSelect(id: string) {
    if (props.modelValue !== undefined) {
        emit('update:modelValue', id);
        const table = timeTables.value.find(t => t.id === id);
        if (table) emit('select', table);
    }
}

function startCreate() {
    isCreating.value = true;
    editingId.value = 'new';
    // Default template
    editForm.value = {
        id: crypto.randomUUID(), // Generate frontend ID, backend might verify
        name: '新时间表',
        periods: [
            { start: "08:00", end: "08:45" },
            { start: "08:55", end: "09:40" },
            { start: "10:00", end: "10:45" },
            { start: "10:55", end: "11:40" }
        ]
    };
}

function startEdit(table: TimeTable) {
    isCreating.value = false;
    editingId.value = table.id;
    // Deep copy
    editForm.value = JSON.parse(JSON.stringify(table));
}

function cancelEdit() {
    editingId.value = null;
    isCreating.value = false;
}

function addPeriod() {
    const last = editForm.value.periods[editForm.value.periods.length - 1];
    let start = "08:00";
    if (last) {
        // Simple logic to add 1 hour
        const [h, _m] = last.end.split(':').map(Number);
        const nextH = h + 1;
        start = `${nextH < 10 ? '0'+nextH : nextH}:${last.end.split(':')[1]}`; // Keep minute
    }
    editForm.value.periods.push({ start, end: start }); // User needs to fix end
}

function removePeriod(index: number) {
    editForm.value.periods.splice(index, 1);
}

async function saveEdit() {
    if (!editForm.value.name) {
        ElMessage.warning('请输入名称');
        return;
    }
    if (editForm.value.periods.length === 0) {
        ElMessage.warning('至少需要一个节次');
        return;
    }
    
    try {
        await saveTimeTable(editForm.value);
        ElMessage.success('保存成功');
        editingId.value = null;
        isCreating.value = false;
    } catch (e) {
        ElMessage.error(`保存失败: ${e}`);
    }
}

async function handleDelete(id: string) {
    try {
        await ElMessageBox.confirm('确定要删除这个时间表吗？', '提示', {
            type: 'warning',
            confirmButtonText: '删除',
            cancelButtonText: '取消'
        });
        await deleteTimeTable(id);
        ElMessage.success('删除成功');
    } catch (e) {
        if (e !== 'cancel') ElMessage.error(`删除失败: ${e}`);
    }
}
</script>

<style scoped>
.time-table-manager {
    height: 100%;
    display: flex;
    flex-direction: column;
}

.header-actions {
    margin-bottom: 12px;
}

.table-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.table-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background-color: var(--surface-color-light);
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid var(--border-color);
}

.table-item:hover {
    background-color: rgba(99, 102, 241, 0.05);
    transform: translateY(-1px);
}

.table-item.is-selected {
    border-color: var(--primary-color);
    background-color: rgba(99, 102, 241, 0.1);
}

.item-content {
    flex: 1;
}

.item-name {
    font-weight: 600;
    color: var(--text-main);
    margin-bottom: 4px;
}

.item-desc {
    font-size: 11px;
    color: var(--text-tertiary);
}

.item-actions {
    display: flex;
    gap: 4px;
}

/* Edit View */
.edit-view {
    display: flex;
    flex-direction: column;
    height: 100%;
}

.edit-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border-color);
}

.edit-title {
    font-weight: 600;
    font-size: 16px;
}

.edit-form {
    flex: 1;
    overflow-y: auto;
    padding-right: 4px;
}

.form-item {
    margin-bottom: 20px;
}

.label {
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: 8px;
    font-weight: 500;
}

.label-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
}

.periods-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.period-row {
    display: flex;
    align-items: center;
    gap: 8px;
}

.period-index {
    width: 24px;
    font-size: 12px;
    color: var(--text-tertiary);
    text-align: center;
}

.time-input {
    flex: 1;
}

.sep {
    color: var(--text-tertiary);
}

.delete-period-btn {
    opacity: 0.5;
}
.delete-period-btn:hover {
    opacity: 1;
}
</style>
