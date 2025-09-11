<template>
  <n-config-provider>
    <n-notification-provider>
      <AutoCheckUpdate />

      <div
        class="flex flex-col overflow-hidden"
        style="height: calc(100vh - 20px)"
      >
        <div class="flex gap-2 p-2 bg-gray-50 border-b flex-shrink-0">
          <n-button
            type="primary"
            :ghost="$route.path !== '/search'"
            @click="changePage('search')"
          >
            搜索页面
          </n-button>

          <n-button
            type="primary"
            :ghost="$route.path !== '/web'"
            @click="changePage('web')"
            class="h-full"
          >
            网页页面
          </n-button>

          <!-- <n-button
            type="primary"
            :ghost="$route.path !== '/manage'"
            @click="changePage('manage')"
            class="h-full"
          >
            管理页面
          </n-button> -->

          <n-button
            type="primary"
            :ghost="$route.path !== '/config'"
            @click="changePage('config')"
            class="h-full"
          >
            配置页面
          </n-button>
        </div>

        <div class="flex-1 overflow-auto">
          <!-- Web 组件始终存在，用于监听事件 -->
          <Web v-show="$route.path === '/web'" />

          <!-- 其他页面的路由视图 -->
          <router-view v-show="$route.path !== '/web'" v-slot="{ Component }">
            <keep-alive include="Search">
              <component :is="Component" />
            </keep-alive>
          </router-view>
        </div>
      </div>
    </n-notification-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import router from "./router";
import Web from "./views/Web.vue";

const changePage = (page: "search" | "config" | "web" | "manage") => {
  router.push(`/${page}`);
};
</script>
