<template>
  <div class="w-96 p-6 rounded-2 shadow-lg mx-auto mt-5">
    <n-form label-width="85" label-placement="left">
      <!-- URL 设置 -->
      <n-form-item label="URL">
        <n-input v-model:value="baseUrl" class="w-full" />
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

      <!-- User-Agent -->
      <n-form-item label="User-Agent">
        <n-input
          v-model:value="userAgent"
          placeholder="请输入 User-Agent"
          class="w-full"
        >
          <template #suffix>
            <n-tooltip
              trigger="hover"
              placement="top-start"
              :show-arrow="true"
              :delay="300"
              :duration="200"
              style="max-width: 300px"
            >
              <template #trigger>
                <n-icon size="20" color="black" class="cursor-help">
                  <QuestionCircle24Regular />
                </n-icon>
              </template>
              <div style="white-space: pre-wrap; line-height: 1.4">
                输入获取Cookie时所使用的User-Agent
              </div>
            </n-tooltip>
          </template>
        </n-input>
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
          placeholder="例如：{{title}} - 第{{index}}话"
          class="w-full"
        >
          <template #suffix>
            <n-tooltip
              trigger="hover"
              placement="top-start"
              :show-arrow="true"
              :delay="300"
              :duration="200"
              style="max-width: 350px"
            >
              <template #trigger>
                <n-icon size="20" color="black" class="cursor-help">
                  <QuestionCircle24Regular />
                </n-icon>
              </template>
              <div style="white-space: pre-line; line-height: 1.4">
                {{ templateTitle.trim() }}
              </div>
            </n-tooltip>
          </template>
        </n-input>
      </n-form-item>

      <!-- 是否添加目录页 -->
      <n-form-item label="添加目录页">
        <n-switch v-model:value="addCatalog" />
      </n-form-item>

      <!-- 是否启动检测更新 -->
      <n-form-item label="启动时检测更新">
        <n-switch v-model:value="autoCheckUpdate" />
      </n-form-item>

      <n-form-item>
        <div class="flex justify-center">
          <n-button @click="showVersionDialog">关于</n-button>
        </div>
      </n-form-item>

      <!-- 保存按钮 -->
      <n-form-item>
        <n-button type="primary" class="w-full" @click="saveConfig">
          保存配置
        </n-button>
      </n-form-item>
    </n-form>

    <!-- 版本信息对话框 -->
    <n-modal v-model:show="showVersionModal">
      <n-card
        style="width: 400px"
        title="版本信息"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
      >
        <div class="version-info">
          <div class="version-item">
            <span class="label">当前版本：</span>
            <span class="value">{{ version || "获取中..." }}</span>
          </div>
          <div class="version-item">
            <span class="label">应用名称：</span>
            <span class="value">epub_download</span>
          </div>
        </div>

        <!-- 对话框中的按钮区域 -->
        <div class="dialog-actions">
          <n-button class="action-btn" @click="checkUpdate">检测更新</n-button>
          <n-button
            class="action-btn"
            tag="a"
            href="https://github.com/kotorimiku/epub_download"
            target="_blank"
          >
            <n-icon size="18" class="mr-1"><GithubIcon /></n-icon>
            GitHub
          </n-button>
        </div>

        <template #footer>
          <div class="flex justify-end">
            <n-button @click="showVersionModal = false">关闭</n-button>
          </div>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h } from "vue";
import { useRunCommand } from "../composables/RunCommand";
import { commands } from "../bindings";
import { useNotify } from "../composables/useNotification";
import QuestionCircle24Regular from "@vicons/fluent/QuestionCircle24Regular";
import {
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NButton,
  NSwitch,
  NModal,
  NCard,
  NIcon,
} from "naive-ui";

const runCommand = useRunCommand();
const notify = useNotify();

const template = ref("");
const baseUrl = ref<string>("https://www.bilinovel.com");
const sleepTime = ref<number>(8);
const cookie = ref<string>("");
const userAgent = ref<string>("");
const output = ref<string>("");
const addCatalog = ref(false);
const autoCheckUpdate = ref(true);
const version = ref<string>("");
const showVersionModal = ref(false);

const templateTitle = `
书籍标题：{{book_title}}
章节标题：{{chapter_title}}
章节编号：{{chapter_number}}
章节编号前补零：{{chapter_number:x}}

输入 0：{{book_title}}-{{chapter_title}}
输入 1：{{book_title}}-[{{chapter_number}}]{{chapter_title}}
输入 2：[{{chapter_number}}]{{chapter_title}}
输入 3：[{{chapter_number:2}}]{{chapter_title}}
`;

const saveConfig = () => {
  runCommand({
    command: () =>
      commands.saveConfig({
        output: output.value,
        template: template.value,
        cookie: cookie.value,
        userAgent: userAgent.value,
        sleepTime: sleepTime.value,
        baseUrl: baseUrl.value,
        addCatalog: addCatalog.value,
        autoCheckUpdate: autoCheckUpdate.value,
      }),
    onSuccess: () => {
      notify.success({ content: "保存成功" });
    },
    errMsg: "保存失败",
  });
};

const checkUpdate = () => {
  runCommand({
    command: commands.checkUpdate,
    onSuccess: (msg: string) => {
      const urlMatch = msg.match(/https?:\/\/[^\s]+/);
      if (urlMatch) {
        const url = urlMatch[0];
        notify.success({
          content: () =>
            h("div", [
              h("div", msg.replace(url, "")),
              h(
                "a",
                {
                  href: url,
                  target: "_blank",
                  style:
                    "color: #18a058; text-decoration: underline; cursor: pointer;",
                },
                url
              ),
            ]),
        });
      } else {
        notify.success({ content: msg });
      }
    },
    errMsg: "检测更新失败",
  });
};

const showVersionDialog = () => {
  showVersionModal.value = true;
};

const GithubIcon = {
  render() {
    return h(
      "svg",
      {
        width: "20",
        height: "20",
        viewBox: "0 0 24 24",
        fill: "currentColor",
        xmlns: "http://www.w3.org/2000/svg",
      },
      [
        h("path", {
          d: "M12 1C5.923 1 1 5.923 1 12c0 4.867 3.149 8.979 7.521 10.436.55.096.756-.233.756-.522 0-.262-.013-1.128-.013-2.049-2.764.509-3.479-.674-3.699-1.292-.124-.317-.66-1.293-1.127-1.554-.385-.207-.936-.715-.014-.729.866-.014 1.485.797 1.691 1.128.99 1.663 2.571 1.196 3.204.907.096-.715.385-1.196.701-1.471-2.448-.275-5.005-1.224-5.005-5.432 0-1.196.426-2.186 1.128-2.956-.111-.275-.496-1.402.11-2.915 0 0 .921-.288 3.024 1.128a10.193 10.193 0 0 1 2.75-.371c.936 0 1.871.123 2.75.371 2.104-1.43 3.025-1.128 3.025-1.128.605 1.513.221 2.64.111 2.915.701.77 1.127 1.747 1.127 2.956 0 4.222-2.571 5.157-5.019 5.432.399.344.743 1.004.743 2.035 0 1.471-.014 2.654-.014 3.025 0 .289.206.632.756.522C19.851 20.979 23 16.854 23 12c0-6.077-4.922-11-11-11Z",
        }),
      ]
    );
  },
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
        userAgent.value = res.userAgent;
        output.value = res.output;
        addCatalog.value = res.addCatalog;
        if (typeof res.autoCheckUpdate === "boolean") {
          autoCheckUpdate.value = res.autoCheckUpdate;
        } else {
          autoCheckUpdate.value = true;
        }
      }
    },
    errMsg: "获取配置失败",
  });

  // 获取版本信息
  runCommand({
    command: commands.getVersion,
    onSuccess: (ver: string) => {
      version.value = `v${ver}`;
    },
    errMsg: "获取版本信息失败",
  });
});
</script>

<style scoped>
.version-info {
  padding: 16px 0;
}

.version-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid #f0f0f0;
}

.version-item:last-child {
  border-bottom: none;
}

.label {
  font-weight: 500;
  color: #666;
}

.value {
  font-weight: 600;
  color: #333;
}

.dialog-actions {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid #f0f0f0;
  display: flex;
  gap: 12px;
  justify-content: center;
}

.action-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
