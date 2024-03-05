<template>
    <!-- TODO:
- splashscreen for guest (show drag fn)
- splashscreen for host (show local ip - with tnc saying if under same network / public ip)
-->
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

            <!-- TODO: progress -->
            <Progress v-model="uploadProgress" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, unref, watch, onMounted } from "vue";
import { useFileDialog, useDropZone, useEventSource } from "@vueuse/core";
import axios from "axios";

import { FileViewModel } from "@/types/File";

import { UploadIcon, DownloadIcon } from "lucide-vue-next";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import FileDownloadTable from "@/components/organisms/FileDownloadTable.vue";

import { toast } from "vue-sonner";

const ip = ref();
onMounted(() => {
    ip.value = new URL(window.location.href).origin;
});

const { data } = useEventSource("/events");
// #region listing
const list = ref<FileViewModel[]>([]);
onMounted(async () => {
    const { statusText, status, code, data } = await axios
        .get(`${unref(ip)}/list`)
        .catch((error) => error);

    if (statusText !== "OK") {
        toast.error(`Failed to get file list!`, {
            description: `Get file list with status: ${status}; code: ${code}`,
        });

        return;
    }

    list.value = Object.entries(data).map(([fileName, id]) => ({
        id,
        fileName,
    }));
});
watch(data, (v) => {
    try {
        const { action, payload } = JSON.parse(v);

        switch (action) {
            case "file-added": {
                const { id, file_name } = JSON.parse(payload);
                if (unref(list).find((v) => v.id === id)) return;

                list.value = [...unref(list), { id, fileName: file_name }];

                break;
            }
            case "all-files-cleared": {
                list.value = [];
                break;
            }
        }
    } catch (err) {}
});

// #endregion

// #region download
const handleDownload = (id: string) => {
    const link = document.createElement("a");
    link.href = `${unref(ip)}/dl/${id}`;
    link.click();
};

const handleDownloadAll = () => {
    const link = document.createElement("a");
    link.href = `${unref(ip)}/dl`;
    link.click();
};
// #endregion

// #region upload
const uploadFiles = async (files: File[]) => {
    if (!Array.isArray(files)) throw new Error("No files are selected.");

    const formData = new FormData();
    files.forEach((file) => {
        formData.append("file", file);
    });

    const response = await axios
        .post(`${unref(ip)}/upload`, formData, {
            headers: {
                "Content-Type": "multipart/form-data",
            },
        })
        .catch((error) => error);

    if (response.statusText !== "OK") {
        toast.error(`Failed Upload!`, {
            description: `Upload failed with status: ${response.status}; code: ${response.code}`,
        });

        return;
    }

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
    onDrop: async (files: File[] | null, event: DragEvent) => {
        const directoryPaths = Object.values(event.dataTransfer?.items ?? {})
            .map((item) => item.webkitGetAsEntry())
            .filter((item) => item?.isDirectory)
            .map((item) => item.name);

        if (directoryPaths.length > 0) {
            toast.error(
                directoryPaths.length
                    ? "Cannot add directory!"
                    : "Cannot add directories",
                {
                    // @ts-ignore
                    description: directoryPaths.join(", "),
                },
            );
            return;
        }

        if (!files) {
            toast.error("No files detected!", {
                // @ts-ignore
                description: error,
            });
            return;
        }

        await uploadFiles(files);
    },
});
onMounted(() => {
    dropZoneRef.value = document.body;
});

const { open: openFileDialog, onChange: onfileDialogFilesChange } =
    useFileDialog({
        multiple: true,
    });
onfileDialogFilesChange(async (files: FileList) => {
    try {
        await uploadFiles([...files]);
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
