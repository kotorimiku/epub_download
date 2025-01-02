<template>
    <div>
        <!-- 书籍 ID 输入框 -->
        <input type="text" v-model="bookId" placeholder="请输入书籍ID" class="input-box" />

        <!-- 按钮 -->
        <button @click="search" class="btn">搜索</button>
        <button @click="download" :disabled="!volumeList.length" class="btn"
            :class="{ disabled: !volumeList.length }">开始下载</button>
        <!-- 全选/反选按钮 -->
        <button @click="selectAll" class="btn">全选</button>
        <button @click="selectInverse" class="btn">反选</button>


        <div class="box">
            <!-- 消息框 -->
            <div class="message-box" ref="messageBox">
                <div v-for="(message, index) in messages" :key="index" class="message-item">
                    {{ message }}
                </div>
            </div>

            <!-- 卷选择框 -->
            <div class="switch-container">
                <p v-if="bookInfo">{{ bookInfo.title }}</p>

                <div v-for="(volume, index) in volumeList" :key="index" class="switch-item">
                    <input type="checkbox" v-model="selectedVolumes" :value="index + 1"
                        :id="'volume-' + index.toString()" @click="toggleVolumeSelection(index)" />
                    <label :for="'volume-' + index.toString()">{{ volume.title }}</label>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core'; // 使用 Tauri 的核心 API
import { listen } from '@tauri-apps/api/event'; // 监听事件

// 数据绑定
const messages = ref<string[]>([]);
const selectedVolumes = ref<number[]>([]); // 用户选择的卷
const bookInfo = ref<any | null>(null);
const volumeList = ref<any[]>([]); // 书籍卷列表
const bookId = ref<string>(""); // 用户输入的书籍 ID
const isDownloading = ref(false); // 是否正在下载

// 滚动框引用
const messageBox = ref<HTMLElement | null>(null);

// 记录上一个选中的卷索引
let lastSelectedIndex = ref<number | null>(null);
let isShiftPressed = ref(false);

// 监听后端事件
listen('message', (event) => {
    messages.value.push(event.payload as string);
    scrollToBottom(); // 滚动到最新消息
});

listen('image', (event) => {
    messages.value.pop();
    messages.value.push(event.payload as string);
    scrollToBottom(); // 滚动到最新消息
});

// 检测 Shift 键按下状态
const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === "Shift") {
        isShiftPressed.value = true;
    }
};

const handleKeyup = (event: KeyboardEvent) => {
    if (event.key === "Shift") {
        isShiftPressed.value = false;
    }
};

// 选择卷
const toggleVolumeSelection = (index: number) => {
    if (isShiftPressed.value && lastSelectedIndex.value !== null) {
        const start = Math.min(lastSelectedIndex.value, index);
        const end = Math.max(lastSelectedIndex.value, index);
        const range = Array.from({ length: end - start + 1 }, (_, i) => start + i);

        // 更新 selectedVolumes
        range.forEach(i => {
            if (selectedVolumes.value.includes(i + 1)) {
                const idx = selectedVolumes.value.indexOf(i + 1);
                if (idx > -1) selectedVolumes.value.splice(idx, 1);
            } else {
                selectedVolumes.value.push(i + 1);
            }
        });
    } else {
        // 单独选择
        const volumeIndex = selectedVolumes.value.indexOf(index + 1);
        if (volumeIndex > -1) {
            selectedVolumes.value.splice(volumeIndex, 1);
        } else {
            selectedVolumes.value.push(index + 1);
        }
    }

    lastSelectedIndex.value = index;
};

// 全选
const selectAll = () => {
    selectedVolumes.value = volumeList.value.map((_, index) => index + 1);
};

// 反选
const selectInverse = () => {
    volumeList.value.forEach((_, index) => {
        const volumeIndex = selectedVolumes.value.indexOf(index + 1);
        if (volumeIndex > -1) {
            selectedVolumes.value.splice(volumeIndex, 1);
        } else {
            selectedVolumes.value.push(index + 1);
        }
    });
};

// 搜索书籍信息
const search = async () => {
    if (!bookId.value.trim()) {
        return;
    }

    try {
        const result: any = await invoke('get_book_info', {
            bookId: bookId.value.trim(),
        });

        bookInfo.value = result[0];
        volumeList.value = result[1];
        selectedVolumes.value = [];

        messages.value.push(`书籍 ${bookInfo.value.title} 信息获取成功！`);
        scrollToBottom();
    } catch (error) {
        console.error(error);
        messages.value.push("书籍信息获取失败，请检查书籍 ID 或网络连接！");
        scrollToBottom();
    }
};

// 下载选中卷
const download = async () => {
    if (selectedVolumes.value.length === 0 || isDownloading.value) {
        return; // 没有选中卷，直接返回
    }
    try {
        isDownloading.value = true;
        await invoke('download', {
            bookId: bookId.value.trim(),
            bookInfo: bookInfo.value,
            volumeList: volumeList.value,
            volumeNoList: selectedVolumes.value,
        });

        messages.value.push("下载任务完成！");
        scrollToBottom();
    } catch (error) {
        console.error(error);
        messages.value.push("下载失败，请重试！");
        scrollToBottom();
    } finally {
        isDownloading.value = false;
    }
};

// 滚动到底部
const scrollToBottom = () => {
    if (messageBox.value) {
        messageBox.value.scrollTop = messageBox.value.scrollHeight;
    }
};

// 初始化时滚动到消息框底部
onMounted(() => {
    scrollToBottom();
    window.addEventListener('keydown', handleKeydown);
    window.addEventListener('keyup', handleKeyup);
});
</script>

<style scoped>
/* 容器布局 */
.box {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    margin-top: 20px;
    background-color: #f9f9f9;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 5px;
}

/* 消息框样式 */
.message-box {
    flex: 2;
    height: 60vh;
    overflow-y: auto;
    border: 1px solid #ccc;
    margin-right: 10px;
    background-color: #fff;
    padding: 10px;
}

/* 卷选择框样式 */
.switch-container {
    flex: 1;
    display: flex;
    height: 60vh;
    flex-direction: column;
    overflow-y: auto;
    border: 1px solid #ccc;
    gap: 5px;
    font-size: 14px;
    padding: 10px;
}

/* 消息条目样式 */
.message-item {
    margin-bottom: 10px;
    word-wrap: break-word;
    font-size: 14px;
    color: #333;
}

/* 输入框样式 */
.input-box {
    width: 250px;
    padding: 8px;
    font-size: 14px;
    border-radius: 5px;
    border: 1px solid #ddd;
    margin-right: 10px;
    margin-bottom: 10px;
}

/* 按钮样式 */
.btn {
    padding: 8px 16px;
    font-size: 14px;
    background-color: #fff;
    border: 1px solid #ddd;
    border-radius: 5px;
    cursor: pointer;
    transition: background-color 0.3s ease;
}

.btn:hover {
    background-color: #f1f1f1;
}

.disabled {
    background-color: #e0e0e0;
    cursor: not-allowed;
}
</style>