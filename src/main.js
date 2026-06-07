import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import "./assets/main.css";
import "./assets/themes/default.css";
import "./assets/themes/reader.css";

const app = createApp(App);

app.use(createPinia());
app.use(router);
app.mount("#app");
