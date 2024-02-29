<template>
    <!-- TODO: mt-32 to only apply on tauri app size -->
    <div class="grow flex flex-col mt-32 mb-4">
        <FileHostingTable
            class="mx-auto w-full"
            :list="list"
            @upload="handleUpload"
            @clear-all="handleClearAll"
        />

        <!-- TODO:  tauri drag drop zone -->
    </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { ref } from "vue";

import { FileViewModel } from "@/types/File";

import FileHostingTable from "@/components/organisms/FileHostingTable.vue";

import { toast } from "vue-sonner";

const list = ref<FileViewModel>([
    {
        id: "m5gr84i9",
        fileName: "gxbank calc.js",
    },
    {
        id: "3u1reuv4",
        fileName: "han's cny 2024 ticket.pdf",
    },
]);

const handleUpload = async () => {
    try {
        const filePaths = await open({
            multiple: true,
        });

        if (!Array.isArray(filePaths)) return;

        console.log("handle upload", filePaths);

        filePaths.forEach((filePath) => {
            invoke("add_file", { path: filePath });
            toast.success("Successful Upload!", {
                description: `Your file "${filePath}" is successfully uploaded.`,
            });
        });
    } catch (error) {
        toast.error("Failed to upload!", {
            // @ts-ignore
            description: error,
        });
    }
};

const handleClearAll = () => {
    toast.success("Successful Clear!", {
        description: `All your files are successfully cleared.`,
    });
    invoke("clear_files");
};
</script>
