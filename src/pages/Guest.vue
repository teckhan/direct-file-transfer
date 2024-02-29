<template>
    i am guest

    <button type="button" @click="openFilePicker">
        upload
    </button>
</template>


<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog';

const openFilePicker = async () => {
    const filePaths = await open({
        multiple: true
    })

    if (!Array.isArray(filePaths)) return

    console.log('gonna upload', filePaths)
}
</script>
