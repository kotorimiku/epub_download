<template>
  <div class="p-x-2 space-y-4 h-full flex flex-col">
    <div class="flex items-center gap-2 h-10">
      <n-input
        v-model:value="url"
        placeholder="输入图片 URL"
        class="w-96"
        @keyup.enter="send('Rustls')"
      />
      <n-button type="primary" @click="send('Rustls')" :loading="loading">请求 rustls</n-button>
      <n-button type="primary" @click="send('NativeTls')" :loading="loading"
        >请求 native-tls</n-button
      >
      <n-button @click="clear">清空</n-button>
    </div>

    <n-card class="flex-1 min-h-0" :content-style="cardContentStyle">
      <div v-if="loading" class="text-gray-600">加载中...</div>

      <div v-else-if="imageSrc" class="image-wrap">
        <img :src="imageSrc" alt="preview" class="preview-image" />
      </div>

      <div v-else-if="error" class="text-red-500">{{ error }}</div>

      <div v-else class="text-gray-400">输入图片 URL 并请求</div>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { Channel } from '@tauri-apps/api/core';
import { computed, onBeforeUnmount, ref } from 'vue';

import { commands } from '../bindings';
import type { Tls } from '../bindings';
import { useRunCommand } from '../composables/useRunCommand';

const url = ref<string>('');
const imageSrc = ref<string>('');
const error = ref<string>('');
const loading = ref<boolean>(false);

const cardContentStyle = computed(() => ({
  height: '100%',
  overflow: 'auto',
}));

const runCommand = useRunCommand();

const send = async (tls: Tls) => {
  if (!url.value.trim()) return;
  loading.value = true;
  if (imageSrc.value) {
    URL.revokeObjectURL(imageSrc.value);
  }
  imageSrc.value = '';
  error.value = '';

  const channel = new Channel<number[]>();

  channel.onmessage = (data) => {
    const bytes = new Uint8Array(data);
    if (imageSrc.value) {
      URL.revokeObjectURL(imageSrc.value);
    }
    const blob = new Blob([bytes], { type: 'image/jpeg' });
    imageSrc.value = URL.createObjectURL(blob);
  };

  await runCommand({
    command: () => commands.requestImg(url.value.trim(), tls, channel),
    onError: (err) => {
      imageSrc.value = '';
      error.value = `请求图片失败: ${err}`;
    },
    onFinally: () => {
      loading.value = false;
    },
  });
};

const clear = () => {
  url.value = '';
  if (imageSrc.value) {
    URL.revokeObjectURL(imageSrc.value);
  }
  imageSrc.value = '';
  error.value = '';
};

onBeforeUnmount(() => {
  if (imageSrc.value) {
    URL.revokeObjectURL(imageSrc.value);
  }
});
</script>

<style scoped>
.image-wrap {
  width: 100%;
  min-height: 100%;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  overflow: auto;
}

.preview-image {
  display: block;
  max-width: 100%;
  height: auto;
}
</style>
