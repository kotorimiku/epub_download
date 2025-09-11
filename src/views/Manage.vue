<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { commands } from "../bindings";
import { useRunCommand } from "../composables/RunCommand";
import { Book } from "../bindings";

const books = ref<Book[]>([]);
const runCommand = useRunCommand();

// 分页相关状态
const currentPage = ref(1);
const pageSize = ref(10);
const total = computed(() => books.value.length);

// 计算当前页显示的书籍
const paginatedBooks = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  const end = start + pageSize.value;
  return books.value.slice(start, end);
});

// 分页控制函数
const handlePageChange = (page: number) => {
  currentPage.value = page;
};

const handlePageSizeChange = (size: number) => {
  pageSize.value = size;
  currentPage.value = 1; // 重置到第一页
};

const getBooks = () => {
  runCommand({
    command: () => commands.getBooks(),
    onSuccess: (result: any) => {
      books.value = result;
    },
  });
};

const createIndex = () => {
  runCommand({
    command: () => commands.createIndex(),
  });
};
onMounted(() => {
  getBooks();
});
</script>

<template>
  <div>
    <n-button @click="createIndex">创建索引</n-button>
    <n-card>
      <template #header>
        <div class="flex justify-between items-center">
          <span>书籍管理 (共 {{ total }} 本)</span>
          <n-select
            v-model:value="pageSize"
            :options="[
              { label: '10条/页', value: 10 },
              { label: '20条/页', value: 20 },
              { label: '50条/页', value: 50 },
              { label: '100条/页', value: 100 },
            ]"
            @update:value="handlePageSizeChange"
            style="width: 120px"
          />
        </div>
      </template>

      <n-list>
        <n-list-item v-for="book in paginatedBooks" :key="book.id">
          <div>{{ book.title }}</div>
        </n-list-item>
      </n-list>

      <template #footer>
        <div class="flex justify-center mt-4">
          <n-pagination
            v-model:page="currentPage"
            :page-count="Math.ceil(total / pageSize)"
            :page-size="pageSize"
            :show-size-picker="false"
            show-quick-jumper
            @update:page="handlePageChange"
          />
        </div>
      </template>
    </n-card>
  </div>
</template>
