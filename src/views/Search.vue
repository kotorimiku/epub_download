<template>
  <div class="p-2 space-y-4">
    <div class="flex items-center gap-2">
      <n-input v-model:value="bookId" placeholder="请输入书籍ID" class="w-64" />
      <n-button type="primary" @click="search">搜索</n-button>
      <n-button :disabled="!volumeList.length" @click="download" type="success">
        开始下载
      </n-button>
      <n-button @click="selectAll">全选</n-button>
      <n-button @click="selectInverse">反选</n-button>
    </div>

    <div class="flex gap-4 h-75vh">
      <div class="flex-1 overflow-y-auto" ref="messageBox">
        <n-card>
          <div
            v-for="(message, index) in messages"
            :key="index"
            class="mb-2 text-sm text-gray-700"
          >
            {{ message }}
          </div>
        </n-card>
      </div>

      <n-card class="w-1/3 overflow-y-auto">
        <div v-if="bookInfo" class="mb-2 font-bold">{{ bookInfo.title }}</div>
        <div
          v-for="(volume, index) in volumeList"
          :key="index"
          class="flex items-center gap-2 mb-1"
        >
          <n-checkbox
            :label="volume.title"
            :value="index + 1"
            :checked="selectedVolumes.includes(index + 1)"
            @update:checked="toggleVolumeSelection(index)"
          />
        </div>
      </n-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { listen } from "@tauri-apps/api/event"; // 监听事件
import { useRunCommand } from "../composables/RunCommand";
import { commands } from "../bindings";

const runCommand = useRunCommand();

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
listen("message", (event) => {
  messages.value.push(event.payload as string);
  scrollToBottom(); // 滚动到最新消息
});

listen("image", (event) => {
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
    range.forEach((i) => {
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

  runCommand({
    command: () => commands.getBookInfo(bookId.value.trim()),
    onSuccess: (result: any) => {
      bookInfo.value = result[0];
      volumeList.value = result[1];
      selectedVolumes.value = [];

      messages.value.push(`书籍 ${bookInfo.value.title} 信息获取成功！`);
      scrollToBottom();
    },
    onError: () => {
      messages.value.push("书籍信息获取失败，请检查书籍 ID 或网络连接！");
      scrollToBottom();
    },
  });
};

// 下载选中卷
const download = async () => {
  if (selectedVolumes.value.length === 0 || isDownloading.value) {
    return; // 没有选中卷，直接返回
  }

  isDownloading.value = true;

  runCommand({
    command: () =>
      commands.download(
        bookId.value.trim(),
        bookInfo.value,
        volumeList.value,
        selectedVolumes.value
      ),
    onSuccess: () => {
      messages.value.push(`下载任务完成！`);
      scrollToBottom();
    },
    onError: () => {
      messages.value.push("下载失败，请重试！");
      scrollToBottom();
    },
  });

  isDownloading.value = false;
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
  window.addEventListener("keydown", handleKeydown);
  window.addEventListener("keyup", handleKeyup);
});
</script>
