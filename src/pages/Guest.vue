<template>
    <!-- TODO: mt-32 to only apply on tauri app size -->
    <div class="grow flex flex-col mt-32 mb-4">
        <FileDownloadTable
            class="mx-auto w-full"
            :list="list"
            @download="handleDownload"
            @downloadAll="handleDownloadAll"
            @upload="handleUpload"
        />

        <!-- TODO:  tauri drag drop zone -->
        <Progress v-model="uploadProgress" />
    </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";

import { FileViewModel } from "@/types/File";

import { Progress } from "@/components/ui/progress";
import FileDownloadTable from "@/components/organisms/FileDownloadTable.vue";

import { toast } from "vue-sonner";

// #region listing
const list = ref<FileViewModel[]>([
    {
        id: "m5gr84i9",
        fileName: "gxbank calc.js",
    },
    {
        id: "3u1reuv4",
        fileName: "han's cny 2024 ticket.pdf",
    },
]);
// #endregion

// #region download
const handleDownload = (id: string) => {
    console.log("download", id);
};

const handleDownloadAll = () => {
    console.log("download all");
};
// #endregion

// #region upload
const handleUpload = async () => {
    try {
        const filePaths = await open({
            multiple: true,
        });

        if (!Array.isArray(filePaths)) return;

        console.log("handle upload", filePaths);

        filePaths.forEach((filePath) =>
            toast.success("Successful Upload!", {
                description: `Your file "${filePath}" is successfully uploaded.`,
            }),
        );
    } catch (error) {
        toast.error("Failed to upload!", {
            // @ts-ignore
            description: error,
        });
    }
};

const uploadProgress = ref(13);
// #endregion
</script>
