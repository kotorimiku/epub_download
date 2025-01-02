<template>
  <div>
    <!-- 配置切换按钮 -->
    <div class="switch-container">
      <button @click="changePage('search')" class="btn">搜索页面</button>
      <button @click="changePage('config')" class="btn">配置页面</button>
    </div>
    <Search v-show="currentPage === 'search'" />
    <Config v-show="currentPage === 'config'" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import Search from './Search.vue';
import Config from './Config.vue';

const currentPage = ref<'search' | 'config'>('search'); // 当前页面状态

// 切换页面
const changePage = (page: 'search' | 'config') => {
  currentPage.value = page;
};
</script>

<style scoped>

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
</style>
