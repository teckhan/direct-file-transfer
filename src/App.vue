<template>
    <main class="container">
        <Host v-if="isHost" />
        <Guest v-else />
    </main>
    <Toaster />
</template>

<script lang="ts" setup>
import { unref, onMounted } from "vue";
import { useDark } from "@vueuse/core";

import { Toaster } from "@/components/ui/sonner";
import Host from "@/pages/Host.vue";
import Guest from "@/pages/Guest.vue";

const isHost = "__TAURI__" in window;
onMounted(() => {
    isHost
        ? document.documentElement.classList.add("is-host")
        : document.documentElement.classList.remove("is-host");
});

const isDark = useDark();
unref(isDark)
    ? document.documentElement.classList.add("dark")
    : document.documentElement.classList.remove("dark");
</script>

<style global>
@import "assets/index.css";

body {
    @apply min-h-screen;
}

body,
#app,
main {
    @apply flex grow flex-col;
}

:root:not(.dark):not(.is-host) {
    --primary: 255.9 33.88% 48.04%;
    --primary-foreground: 0 0% 100%;
}

html.dark:not(.is-host) {
    --primary: 257.27 100% 87.06%;
    --primary-foreground: 259.06 58.62% 28.43%;
}

:root:not(.dark).is-host {
    --primary: 22.04 39.2% 24.51%;
    --primary-foreground: 0 0% 100%;
}

html.dark.is-host {
    --primary: 21.18 89.47% 96.27%;
    --primary-foreground: 0 0% 6.67%;
}
</style>
