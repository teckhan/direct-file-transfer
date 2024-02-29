<template>
    <div class="container">
        <Host v-if="isHost" />
        <Guest v-else />
    </div>
</template>

<script lang="ts" setup>
import Host from "./pages/Host.vue"
import Guest from "./pages/Guest.vue"

const isHost = '__TAURI__' in window
</script>
