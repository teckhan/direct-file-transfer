<template>
    <main class="container">
        <Host v-if="isHost" />
        <Guest v-else />
    </main>
    <Toaster />
</template>

<script lang="ts" setup>
import { Toaster } from "@/components/ui/sonner";
import Host from "./pages/Host.vue";
import Guest from "./pages/Guest.vue";

const isHost = "__TAURI__" in window;
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
</style>
