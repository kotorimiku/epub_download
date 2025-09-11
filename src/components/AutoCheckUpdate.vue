<template></template>

<script setup lang="ts">
import { onMounted, h } from "vue";
import { commands } from "../bindings";
import { useNotify } from "../composables/useNotification";

const notify = useNotify();

onMounted(async () => {
  const res = await commands.getConfigVue();
  if (res.status === "ok" && res.data.autoCheckUpdate !== false) {
    const msgRes = await commands.checkUpdate();
    if (msgRes.status === "ok") {
      const msg = msgRes.data;
      if (msg.includes("已是最新版本")) {
        return;
      }
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
          duration: 0,
        });
      } else {
        notify.success({ content: msg });
      }
    }
  }
});
</script>
