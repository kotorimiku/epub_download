<script setup lang="ts">
  import { ref, onMounted } from 'vue';
  import { restoreHtml } from '../composables/event';
  import { emit } from '@tauri-apps/api/event';

  const iframe = ref<HTMLIFrameElement | null>(null);

  onMounted(() => {
    iframe.value!.onload = () => {
      // 移除所有display: none的元素
      iframe
        .value!.contentDocument!.getElementById('acontent')!.querySelectorAll('*')
        .forEach(
          (el) => getComputedStyle(el).display === 'none' && el.remove()
        );

      emit(
        'restoreHtml',
        iframe.value?.contentDocument?.documentElement?.outerHTML
      );
    };
    restoreHtml((html) => {
      iframe.value!.srcdoc = html;
    });
  });
</script>

<template>
  <iframe ref="iframe" class="w-full h-full border-none"></iframe>
</template>
