<template>
    i am host

    <button type="button" @click="openFilePicker">
        upload
    </button>
    <button type="button" @click="clear">
        clear
    </button>
</template>


<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

const openFilePicker = async () => {
    const filePaths = await open({
        multiple: true
    })

    if (!Array.isArray(filePaths)) return

    filePaths?.forEach(filePath => invoke("add_file", { path: filePath }))
}
const clear = () => {
    invoke("clear_files")
}
</script>
