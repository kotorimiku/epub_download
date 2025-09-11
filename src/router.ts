import { createRouter, createWebHashHistory } from "vue-router";
import Search from "./views/Search.vue";
import Config from "./views/Config.vue";
import Manage from "./views/Manage.vue";

const routes = [
  {
    path: "/",
    redirect: "/search",
  },
  {
    path: "/search",
    component: Search,
  },
  {
    path: "/config",
    component: Config,
  },
  {
    path: "/manage",
    component: Manage,
  },
  {
    path: "/web",
    component: { template: '<div></div>' }, // 空组件，因为 Web 组件在 App.vue 中单独处理
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
