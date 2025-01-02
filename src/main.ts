import { createApp } from "vue";
import App from "./App.vue";
import Toast, { PluginOptions } from "vue-toastification";
// Import the CSS or use your own!
import "vue-toastification/dist/index.css";

const app = createApp(App);
const options: PluginOptions = {
    // You can set your default options here
};

app.use(Toast, options);

app.mount("#app");
