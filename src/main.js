import { createApp } from "vue"
import { createPinia } from "pinia"
import App from "./App.vue"
import vuetify from "./plugins/vuetify"
import dialogDrag from "./directives/dialogDrag"
import { vMaska } from "maska/vue"
import "./styles.css"

createApp(App)
    .use(createPinia())
    .use(vuetify)
    .directive("dialog-drag", dialogDrag)
    .directive("maska", vMaska)
    .mount("#app")