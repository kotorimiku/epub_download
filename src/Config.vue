<template>
  <div class="w-96 p-6 bg-white rounded-2xl shadow-lg mx-auto space-y-5">
    <n-form :label-width="100" label-placement="left">
      <!-- URL 设置 -->
      <n-form-item label="URL">
        <n-button @click="baseUrlChange" class="w-full text-left truncate">
          {{ baseUrl }}
        </n-button>
      </n-form-item>

      <!-- 下载间隔 -->
      <n-form-item label="下载间隔">
        <n-input-number
          v-model:value="sleepTime"
          placeholder="请输入下载间隔（秒）"
          class="w-full"
        />
      </n-form-item>

      <!-- Cookie -->
      <n-form-item label="Cookie">
        <n-input
          v-model:value="cookie"
          placeholder="请输入 Cookie"
          type="textarea"
          class="w-full"
        />
      </n-form-item>

      <!-- 保存路径 -->
      <n-form-item label="保存路径">
        <n-input
          v-model:value="output"
          placeholder="请输入保存路径"
          class="w-full"
        />
      </n-form-item>

      <!-- 命名模板 -->
      <n-form-item label="命名模板">
        <n-input
          v-model:value="template"
          type="text"
          :title="templateTitle"
          placeholder="例如：{{title}} - 第{{index}}话"
          class="w-full"
        />
      </n-form-item>

      <!-- 是否添加目录页 -->
      <n-form-item label="添加目录页">
        <n-switch v-model:value="addCatalog" />
      </n-form-item>

      <!-- 保存按钮 -->
      <n-form-item>
        <n-button type="primary" class="w-full" @click="saveConfig">
          保存配置
        </n-button>
      </n-form-item>
    </n-form>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRunCommand } from "./composables/RunCommand";
import { commands } from "./bindings";
import { useNotify } from "./composables/useNotification";
import {
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NButton,
  NSwitch
} from "naive-ui";

const runCommand = useRunCommand();
const notify = useNotify();

const template = ref("");
const baseUrl = ref<string>("https://www.bilinovel.com");
const sleepTime = ref<number>(8);
const cookie = ref<string>("");
const output = ref<string>("");
const addCatalog = ref(false);

const templateTitle = `
书籍标题使用{{book_title}}，章节标题使用{{chapter_title}}，
章节编号使用{{chapter_number}}，章节编号前填0使用{{chapter_number:x}}，
输入 0 ，使用{{book_title}}-{{chapter_title}}，
输入 1 ，使用{{book_title}}-[{{chapter_number}}]{{chapter_title}}，
输入 2 ，使用[{{chapter_number}}]{{chapter_title}}，
输入 3 ，使用[{{chapter_number:2}}]{{chapter_title}}`;

const baseUrlChange = () => {
  baseUrl.value =
    baseUrl.value === "https://www.bilinovel.com"
      ? "https://tw.linovelib.com"
      : "https://www.bilinovel.com";
};

const saveConfig = () => {
  runCommand({
    command: commands.saveConfig,
    args: [
      {
        output: output.value,
        template: template.value,
        cookie: cookie.value,
        sleepTime: sleepTime.value,
        baseUrl: baseUrl.value
      }
    ],
    onSuccess: () => {
      notify.success({ content: "保存成功" });
    },
    errMsg: "保存失败"
  });
};

onMounted(() => {
  runCommand({
    command: commands.getConfigVue,
    onSuccess: (res: any) => {
      if (res) {
        template.value = res.template;
        baseUrl.value = res.baseUrl;
        sleepTime.value = res.sleepTime;
        cookie.value = res.cookie;
        output.value = res.output;
      }
      if (cookie.value === "") {
        notify.error({ content: "您还没配置cookie，请先配置 Cookie" });
      }
    },
    errMsg: "获取配置失败"
  });
});
</script>
