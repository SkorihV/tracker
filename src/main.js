import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import vuetify from "./plugins/vuetify";
import dialogDrag from "./directives/dialogDrag";
import "./styles.css";

createApp(App)
    .use(createPinia())
    .use(vuetify)
    .directive("dialog-drag", dialogDrag)
    .mount("#app");