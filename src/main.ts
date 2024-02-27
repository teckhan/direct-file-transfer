import { createApp } from "vue";
import "./styles.css";
import App from "./App.vue";

fetch("https://checkip.amazonaws.com/", { mode: "no-cors" })
    .then((res) => {
        console.log(res);
        return res.text();
    })
    .then((data) => console.log("aws", data))
    .catch((error) => console.error("Error:", error));

// TODO: condition here to load client vs host
createApp(App).mount("#app");
