<script setup lang="ts">
import { ref, onMounted } from "vue";
import { restoreHtml } from "../composables/event";
import { emit } from "@tauri-apps/api/event";

const iframe = ref<HTMLIFrameElement | null>(null);

let restore: void | (() => void) | null = null;

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
    // 移除所有display: none的元素
    iframe
      .value!.contentDocument!.getElementById("acontent")!
      .querySelectorAll("*")
      .forEach((el) => getComputedStyle(el).display === "none" && el.remove());

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
  <div class="w-full h-full overflow-hidden">
    <iframe
      ref="iframe"
      referrerpolicy="no-referrer"
      class="w-full h-full border-none"
    ></iframe>
  </div>
</template>
