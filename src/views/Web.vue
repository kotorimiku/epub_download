<script setup lang="ts">
import { ref, onMounted } from "vue";
import { restoreHtml } from "../composables/event";
import { emit } from "@tauri-apps/api/event";
import { useRunCommand } from "@/composables/RunCommand";
import { commands } from "@/bindings";
import { NInput, NButton } from "naive-ui";
import { globalStore } from "@/store/global";

const iframe = ref<HTMLIFrameElement | null>(null);
const url = ref<string>("");

let restore: void | (() => void) | null = null;

const runCommand = useRunCommand();

const getHtml = (url: string) => {
  if (!url) return;
  if (globalStore.isDownloading) return;
  runCommand({
    command: () => commands.browserUrl(url),
    onSuccess: (result: any) => {
      iframe.value!.srcdoc = result;
    },
  });
};

onMounted(() => {
  // 重写iframe的srcdoc属性，自动注入修改navigator.platform的脚本
  const originalSrcdocDescriptor = Object.getOwnPropertyDescriptor(
    HTMLIFrameElement.prototype,
    "srcdoc"
  );

  Object.defineProperty(iframe.value!, "srcdoc", {
    set: function (html: string) {
      // 在HTML头部注入脚本，在其他脚本执行前修改navigator.platform
      const scriptTag = `<script>
        Object.defineProperty(navigator, "platform", {
          get: function () {
            return "android";
          },
          configurable: true,
        });
      <\/script>`;

      // 在head标签后插入脚本，如果没有head标签则在html标签后插入
      let modifiedHtml = html;
      if (html.includes("<head>")) {
        modifiedHtml = html.replace("<head>", `<head>${scriptTag}`);
      } else if (html.includes("<html>")) {
        modifiedHtml = html.replace(
          "<html>",
          `<html><head>${scriptTag}</head>`
        );
      } else {
        modifiedHtml = `<html><head>${scriptTag}</head><body>${html}</body></html>`;
      }

      // 调用原始的srcdoc setter
      originalSrcdocDescriptor?.set?.call(this, modifiedHtml);
    },
    get: function () {
      return originalSrcdocDescriptor?.get?.call(this);
    },
    configurable: true,
    enumerable: true,
  });

  iframe.value!.onload = () => {
    iframe
      .value!.contentDocument!.getElementById("acontent")!
      .querySelectorAll("*")
      .forEach((el) => {
        const style = getComputedStyle(el);
        if (
          style.display === "none" ||
          style.transform === "matrix(0, 0, 0, 0, 0, 0)" ||
          style.position === "absolute"
        ) {
          console.log(el);
          el.remove();
        }
        return;
      });

    emit(
      "restoreHtml",
      iframe.value?.contentDocument?.documentElement?.outerHTML
    );
  };

  restore = restoreHtml((html) => {
    iframe.value!.srcdoc = html;
  });
});

onUnmounted(() => {
  restore?.();
});
</script>

<template>
  <div class="w-full h-full overflow-hidden flex flex-col">
    <div class="flex items-center gap-2 h-10 px-2">
      <n-input
        v-model:value="url"
        placeholder="请输入URL"
        class="w-full"
        @keyup.enter="getHtml(url)"
      />
      <n-button
        type="primary"
        @click="getHtml(url)"
        :disabled="globalStore.isDownloading"
        >浏览</n-button
      >
    </div>

    <iframe
      ref="iframe"
      referrerpolicy="no-referrer"
      class="w-full h-full border-none flex-1"
    ></iframe>
  </div>
</template>
