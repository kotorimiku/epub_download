<template>
  <n-notification-provider>
    <div class="p-2 ">
      <!-- 页面切换按钮区域 -->
      <div class="flex  gap-2">
        <n-button
          type="primary"
          :ghost="currentPage !== 'search'"
          @click="changePage('search')"
        >
          搜索页面
        </n-button>
        <n-button
          type="primary"
          :ghost="currentPage !== 'config'"
          @click="changePage('config')"
        >
          配置页面
        </n-button>
      </div>

      <!-- 页面内容 -->
      <Search v-show="currentPage === 'search'" />
      <Config v-show="currentPage === 'config'" />
    </div>
  </n-notification-provider>
</template>

<script setup lang="ts">
import { ref } from "vue";

const currentPage = ref<"search" | "config">("search");

const changePage = (page: "search" | "config") => {
  currentPage.value = page;
};
</script>
