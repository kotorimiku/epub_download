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
      <n-card class="flex-1">
        <template #cover>
          <div ref="messageBox" class="h-full overflow-y-auto ml-5">
            <div
              v-for="(message, index) in messages"
              :key="index"
              class="mb-2 text-sm text-gray-700"
            >
              {{ message }}
            </div>
          </div>
        </template>
      </n-card>

      <n-card class="volume w-1/3 overflow-y-auto">
        <selection-area
          class="container"
          :options="{ selectables: '.selectable', boundaries: '.volume' } as SelectionOptions"
          :onMove="onMove"
          :onStart="onStart"
        >
          <n-checkbox
            class="flex items-center gap-2 mb-1 selectable hover:bg-gray-200!"
            v-for="(volume, index) in volumeList"
            :key="index"
            :data-key="index + 1"
            :label="volume.title"
            value="index + 1"
            :checked="selectedVolumes.has(index + 1)"
          />
        </selection-area>
        <template #header>
          <div v-if="bookInfo" class="font-bold">
            {{ bookInfo.title }}
          </div>
        </template>
        <template #footer></template>
      </n-card>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, onMounted, nextTick } from 'vue';
  import { listen } from '@tauri-apps/api/event'; // 监听事件
  import { useRunCommand } from '../composables/RunCommand';
  import { commands } from '../bindings';
  import {
    SelectionArea,
    SelectionEvent,
    SelectionOptions,
  } from '@viselect/vue';

  const extractIds = (els: Element[]): number[] => {
    return els
      .map((v) => v.getAttribute('data-key'))
      .filter(Boolean)
      .map(Number);
  };
  const onStart = ({ event, selection }: SelectionEvent) => {
    if (!event?.ctrlKey && !event?.metaKey) {
      selection.clearSelection();
      selectedVolumes.value.clear();
    }
  };

  const onMove = ({
    store: {
      changed: { added, removed },
    },
  }: SelectionEvent) => {
    extractIds(added).forEach((id) => selectedVolumes.value.add(id));
    extractIds(removed).forEach((id) => selectedVolumes.value.delete(id));
  };

  const runCommand = useRunCommand();

  // 数据绑定
  const messages = ref<string[]>([]);
  const selectedVolumes = ref<Set<number>>(new Set<number>()); // 用户选择的卷
  selectedVolumes.value.add(1);
  const bookInfo = ref<any | null>(null);
  const volumeList = ref<any[]>([]); // 书籍卷列表
  const bookId = ref<string>(''); // 用户输入的书籍 ID
  const isDownloading = ref(false); // 是否正在下载

  // 滚动框引用
  const messageBox = ref<HTMLDivElement | null>(null);

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

  // 全选
  const selectAll = () => {
    selectedVolumes.value = new Set<number>(
      Array.from({ length: volumeList.value.length }, (_, index) => index + 1)
    );
    console.log(selectedVolumes.value);
  };

  // 反选
  const selectInverse = () => {
    selectedVolumes.value = new Set(
      volumeList.value
        .map((_, index) => index + 1)
        .filter((_, index) => !selectedVolumes.value.has(index + 1))
    );
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
        selectedVolumes.value = new Set();

        messages.value.push(`书籍 ${bookInfo.value.title} 信息获取成功！`);
        scrollToBottom();
      },
      onError: () => {
        messages.value.push('书籍信息获取失败，请检查书籍 ID 或网络连接！');
        scrollToBottom();
      },
    });
  };

  // 下载选中卷
  const download = async () => {
    if (selectedVolumes.value.size === 0 || isDownloading.value) {
      return; // 没有选中卷，直接返回
    }

    isDownloading.value = true;

    runCommand({
      command: () =>
        commands.download(
          bookId.value.trim(),
          bookInfo.value,
          volumeList.value,
          Array.from(selectedVolumes.value)
        ),
      onSuccess: () => {
        messages.value.push(`下载任务完成！`);
        scrollToBottom();
      },
      onError: () => {
        messages.value.push('下载失败，请重试！');
        scrollToBottom();
      },
    });

    isDownloading.value = false;
  };

  // 滚动到底部
  const scrollToBottom = () => {
    nextTick(() => {
      if (messageBox.value) {
        console.log(messageBox.value.scrollHeight);

        messageBox.value.scrollTop = messageBox.value.scrollHeight;

        console.log(messageBox.value.scrollTop);
      }
    });
  };

  // 初始化时滚动到消息框底部
  onMounted(() => {
    scrollToBottom();
  });
</script>

<style>
  .selection-area {
    background: rgba(46, 115, 252, 0.11);
    border: 1px solid rgba(98, 155, 255, 0.85);
    border-radius: 0.15em;
  }
</style>
