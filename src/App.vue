<template>
  <n-config-provider>
    <n-notification-provider>
      <AutoCheckUpdate />

      <div class="flex flex-col">
        <div class="flex gap-2 p-2 bg-gray-50 border-b shrink-0">
          <n-button
            type="primary"
            :ghost="currentPage !== 'search'"
            @click="changePage('search')"
          >
            搜索页面
          </n-button>

          <n-button
            type="primary"
            :ghost="currentPage !== 'web'"
            @click="changePage('web')"
          >
            网页页面
          </n-button>
          <n-button
            type="primary"
            :ghost="currentPage !== 'config'"
            @click="changePage('config')"
          >
            配置页面
          </n-button>
        </div>

        <div class="h-89vh">
          <Search v-show="currentPage === 'search'" />
          <Config v-show="currentPage === 'config'" />
          <Web v-show="currentPage === 'web'" />
        </div>
      </div>
    </n-notification-provider>
  </n-config-provider>
</template>


<script setup lang="ts">
  import { ref } from 'vue';

  const currentPage = ref<'search' | 'config' | 'web'>('search');

  const changePage = (page: 'search' | 'config' | 'web') => {
    currentPage.value = page;
  };
</script>
