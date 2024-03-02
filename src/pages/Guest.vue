<template>
    <div class="grow flex flex-col">
        <Transition
            enter-from-class="scale-105 opacity-80"
            leave-active-class="opacity-0"
        >
            <section
                v-show="doShowDragAndDrop"
                class="cursor-grabbing select-none fixed inset-0 z-10 flex flex-col bg-black transition ease-in"
            >
                <div class="p-8 grow flex flex-col pointer-events-none">
                    <div
                        class="grow flex flex-col p-4 border-4 border-white border-dashed rounded-lg"
                    >
                        <UploadIcon
                            class="animate-[appear-from-inside_300ms_150ms_ease-in-out_backwards] m-auto w-full h-full max-w-20"
                        />
                    </div>
                </div>
            </section>
        </Transition>

        <div class="flex flex-col m-auto sm:mt-32 sm:mb-4 w-full">
            <div class="flex py-4 items-center">
                <div
                    class="grid sm:grid-flow-col gap-2 items-center w-full sm:ml-auto sm:w-auto"
                >
                    <Button
                        class="order-2 sm:order-none"
                        variant="secondary"
                        size="lg"
                        @click="openFileDialog"
                    >
                        <UploadIcon class="mr-2 h-4 w-4" />
                        Upload
                    </Button>
                    <Button
                        size="lg"
                        :disabled="list.length === 0"
                        @click="handleDownloadAll"
                    >
                        <DownloadIcon class="mr-2 h-4 w-4" />
                        Download All
                    </Button>
                </div>
            </div>

            <FileDownloadTable
                class="mx-auto w-full"
                :list="list"
                @download="handleDownload"
            />

            <!-- TODO:  tauri drag drop zone -->
            <Progress v-model="uploadProgress" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useFileDialog, useDropZone } from "@vueuse/core";

import { FileViewModel } from "@/types/File";

import { UploadIcon, DownloadIcon } from "lucide-vue-next";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import FileDownloadTable from "@/components/organisms/FileDownloadTable.vue";

import { toast } from "vue-sonner";

// #region listing
const list = ref<FileViewModel[]>([]);
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
const uploadFiles = (files: File[]) => {
    if (!Array.isArray(files)) throw new Error("No files are selected.");
    // if (files.some((file) => file.type))
    //     // TODO: type is not correcrt indicator
    //     throw new Error("Cannot add directory!"); // TODO: show which name

    files.forEach((file) => {
        console.log("upload", file);
    });

    if (files.length === 1) {
        toast.success("Successful Upload!", {
            description: `Your file "${files[0].name}" is successfully uploaded.`,
        });
    } else {
        toast.success("Successful Uploads!", {
            description: `Your files are successfully uploaded: ${files.map((file) => file.name).join(", ")}`,
        });
    }
};

const dropZoneRef = ref<HTMLElement>();
const { isOverDropZone: doShowDragAndDrop } = useDropZone(dropZoneRef, {
    onDrop: (files: File[] | null) => {
        if (!files) return;
        uploadFiles(files);
    },

    // TODO: allow multiple drag
});
onMounted(() => {
    dropZoneRef.value = document.body;
});

const { open: openFileDialog, onChange: onfileDialogFilesChange } =
    useFileDialog({
        multiple: true,
    });
onfileDialogFilesChange((files: FileList) => {
    try {
        console.log([...files]);
        uploadFiles([...files]);
    } catch (error) {
        toast.error("Failed to upload files!", {
            // @ts-ignore
            description: error,
        });
    }
});

const uploadProgress = ref(13);
// #endregion
</script>
