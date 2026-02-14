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
const pendingRequestId = ref<string>("");

let restore: (() => void) | null = null;

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

onMounted(async () => {
  const waitForAcontentRestored = async (doc: Document): Promise<void> => {
    const target = doc.getElementById("acontent");
    if (!target) return;

    const maxWaitMs = 4000;
    const settleQuietMs = 220;
    const minObserveAfterContentMs = 400;
    const hasRestoreMarker = () => target.querySelector("p[data-k]") !== null;
    const hasVisibleContent = () => {
      const text = target.textContent?.trim() ?? "";
      return text.length > 20;
    };

    if (hasRestoreMarker()) return;

    await new Promise<void>((resolve) => {
      let resolved = false;
      let lastMutationAt = Date.now();
      let contentSeenAt: number | null = hasVisibleContent()
        ? Date.now()
        : null;

      const shouldFinish = () => {
        if (hasRestoreMarker()) return true;
        if (contentSeenAt === null) return false;

        const now = Date.now();
        const observedEnough = now - contentSeenAt >= minObserveAfterContentMs;
        const quietEnough = now - lastMutationAt >= settleQuietMs;
        return observedEnough && quietEnough;
      };

      const finish = () => {
        if (resolved) return;
        resolved = true;
        observer.disconnect();
        window.clearInterval(probeTimer);
        window.clearTimeout(timeoutTimer);
        resolve();
      };

      const observer = new MutationObserver(() => {
        lastMutationAt = Date.now();
        if (contentSeenAt === null && hasVisibleContent()) {
          contentSeenAt = Date.now();
        }

        if (shouldFinish()) {
          finish();
        }
      });

      observer.observe(target, {
        attributes: true,
        attributeFilter: ["data-k"],
        childList: true,
        subtree: true,
        characterData: true,
      });

      const probeTimer = window.setInterval(() => {
        if (contentSeenAt === null && hasVisibleContent()) {
          contentSeenAt = Date.now();
        }

        if (shouldFinish()) {
          finish();
        }
      }, 80);

      const timeoutTimer = window.setTimeout(finish, maxWaitMs);
    });
  };

  // 重写iframe的srcdoc属性，自动注入修改navigator.platform的脚本
  const originalSrcdocDescriptor = Object.getOwnPropertyDescriptor(
    HTMLIFrameElement.prototype,
    "srcdoc",
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
          `<html><head>${scriptTag}</head>`,
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

  iframe.value!.onload = async () => {
    const doc = iframe.value?.contentDocument;
    if (!doc) return;

    const win = doc.defaultView;
    win?.dispatchEvent(new Event("scroll"));
    win?.dispatchEvent(new Event("wheel"));
    doc.dispatchEvent(new KeyboardEvent("keydown", { key: "PageDown" }));

    await waitForAcontentRestored(doc);

    doc
      .getElementById("acontent")
      ?.querySelectorAll("*")
      .forEach((el) => {
        const style = getComputedStyle(el);
        if (
          style.display === "none" ||
          style.transform === "matrix(0, 0, 0, 0, 0, 0)" ||
          style.position === "absolute"
        ) {
          el.remove();
        }
        return;
      });

    if (!pendingRequestId.value) return;

    await emit("restoreHtml", {
      requestId: pendingRequestId.value,
      html: doc.documentElement?.outerHTML ?? "",
    });
    pendingRequestId.value = "";
  };

  restore = await restoreHtml(({ requestId, html }) => {
    pendingRequestId.value = requestId;
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
