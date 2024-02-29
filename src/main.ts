import { internalIpV4 } from "internal-ip";
import { createApp } from "vue";
import "./styles.css";
import App from "./App.vue";

internalIpV4().then((v) => console.log("internal ip", v));

// TODO: condition here to load client vs host
createApp(App).mount("#app");
