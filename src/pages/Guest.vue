<template>
    <div class="grow flex flex-col">
        <Transition
            enter-from-class="scale-105 opacity-80"
            leave-active-class="opacity-0"
        >
            <section
                v-if="isUploading"
                class="select-none fixed inset-0 z-10 flex flex-col bg-background/80 transition ease-in"
            >
                <div
                    class="container p-8 grow flex flex-col pointer-events-none"
                >
                    <div class="m-auto space-y-8 w-full text-center">
                        <h1
                            class="text-4xl font-extrabold tracking-tight lg:text-5xl"
                        >
                            Uploading...
                        </h1>
                        <ul
                            class="mx-auto max-w-xl max-h-64 overflow-y-auto space-y-3 text-left"
                        >
                            <li
                                v-for="[name, percentage] in uploadEntries"
                                :key="name"
                                class="space-y-1"
                            >
                                <div class="flex justify-between gap-4 text-sm">
                                    <span class="truncate">{{ name }}</span>
                                    <span class="whitespace-nowrap"
                                        >{{ percentage }}%</span
                                    >
                                </div>
                                <Progress :model-value="percentage" />
                            </li>
                        </ul>
                    </div>
                </div>
            </section>
        </Transition>

        <Transition
            enter-from-class="scale-105 opacity-80"
            leave-active-class="opacity-0"
        >
            <section
                v-show="doShowDragAndDrop"
                class="cursor-grabbing select-none fixed inset-0 z-10 flex flex-col bg-background transition ease-in"
            >
                <div class="p-8 grow flex flex-col pointer-events-none">
                    <div
                        class="grow flex flex-col p-4 border-4 border-white border-dashed rounded-lg"
                    >
                        <UploadIcon
                            class="animate-appear-from-inside m-auto w-full h-full max-w-20"
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
                        class="order-2 sm:order-0"
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
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, unref, computed, watch, onMounted } from "vue";
import { useFileDialog, useDropZone, useEventSource } from "@vueuse/core";
import axios from "axios";

import { FileViewModel } from "@/types/File";

import { UploadIcon, DownloadIcon } from "@lucide/vue";
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
interface SharedFileDto {
    id: string;
    file_name: string;
    size: number;
}

const list = ref<FileViewModel[]>([]);
onMounted(async () => {
    try {
        const { data } = await axios.get<SharedFileDto[]>(`${unref(ip)}/list`);
        list.value = data.map(({ id, file_name, size }) => ({
            id,
            fileName: file_name,
            size,
        }));
    } catch (error) {
        toast.error("Failed to get file list!", {
            description: String(error),
        });
    }
});
watch(data, (v) => {
    if (!v) return;

    try {
        const { action, payload } = JSON.parse(v);

        switch (action) {
            case "file-added": {
                const { id, file_name, size } = JSON.parse(payload);
                if (unref(list).find((entry) => entry.id === id)) return;

                list.value = [
                    ...unref(list),
                    { id, fileName: file_name, size },
                ];

                break;
            }
            case "file-removed": {
                const { id } = JSON.parse(payload);
                list.value = unref(list).filter((entry) => entry.id !== id);
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
const isUploading = ref(false);
const uploadProgresses = ref<{ [name: string]: number }>({});
const uploadEntries = computed(() =>
    Object.entries(unref(uploadProgresses)),
);
// one request per file so each row in the overlay gets its own real progress
const uploadFiles = async (files: File[]) => {
    if (!Array.isArray(files) || files.length === 0) {
        throw new Error("No files are selected.");
    }

    isUploading.value = true;
    uploadProgresses.value = Object.fromEntries(
        files.map((file) => [file.name, 0]),
    );

    const results = await Promise.allSettled(
        files.map(async (file) => {
            const formData = new FormData();
            formData.append("file", file);

            await axios.post(`${unref(ip)}/upload`, formData, {
                onUploadProgress: (progressEvent) => {
                    uploadProgresses.value = {
                        ...unref(uploadProgresses),
                        [file.name]: Math.round(
                            (progressEvent.progress ?? 0) * 100,
                        ),
                    };
                },
            });

            return file.name;
        }),
    );

    isUploading.value = false;
    uploadProgresses.value = {};

    const succeeded = results
        .filter(
            (result): result is PromiseFulfilledResult<string> =>
                result.status === "fulfilled",
        )
        .map((result) => result.value);
    const failed = files
        .map((file) => file.name)
        .filter((name) => !succeeded.includes(name));

    if (failed.length > 0) {
        toast.error(failed.length === 1 ? "Failed Upload!" : "Failed Uploads!", {
            description: `Could not upload: ${failed.join(", ")}`,
        });
    }

    if (succeeded.length === 1) {
        toast.success("Successful Upload!", {
            description: `Your file "${succeeded[0]}" is successfully uploaded.`,
        });
    } else if (succeeded.length > 1) {
        toast.success("Successful Uploads!", {
            description: `Your files are successfully uploaded: ${succeeded.join(", ")}`,
        });
    }
};

const dropZoneRef = ref<HTMLElement>();
const { isOverDropZone: doShowDragAndDrop } = useDropZone(dropZoneRef, {
    onDrop: async (files: File[] | null, event: DragEvent) => {
        const directoryPaths = Object.values(event.dataTransfer?.items ?? {})
            .map((item) => item.webkitGetAsEntry())
            .filter((item): item is FileSystemEntry => !!item?.isDirectory)
            .map((item) => item.name);

        if (directoryPaths.length > 0) {
            toast.error(
                directoryPaths.length === 1
                    ? "Cannot add directory!"
                    : "Cannot add directories!",
                {
                    description: directoryPaths.join(", "),
                },
            );
            return;
        }

        if (!files || files.length === 0) {
            toast.error("No files detected!");
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
onfileDialogFilesChange(async (files: FileList | null) => {
    if (!files) return;

    try {
        await uploadFiles([...files]);
    } catch (error) {
        toast.error("Failed to upload files!", {
            description: String(error),
        });
    }
});
// #endregion
</script>
