<template>
    <div class="grow flex flex-col">
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
                        class="grow flex flex-col p-4 border-4 border-foreground border-dashed rounded-lg"
                    >
                        <UploadIcon
                            class="animate-[appear-from-inside_300ms_150ms_ease-in-out_backwards] m-auto w-full h-full max-w-20"
                        />
                    </div>
                </div>
            </section>
        </Transition>

        <div
            class="sticky top-[env(safe-area-inset-top,0)] py-8 flex space-x-5 bg-[hsl(var(--background))]"
        >
            <div class="flex w-full items-end gap-2.5">
                <div class="grid gap-2.5 w-full gap-1.5">
                    <Label for="publicIp">Public Ip</Label>
                    <Input
                        id="publicIp"
                        type="text"
                        placeholder="Public Ip"
                        :value="publicIp"
                        readonly
                    />
                </div>
                <div>
                    <Button variant="secondary" size="icon">
                        <ClipboardIcon className="h-4 w-4" />
                    </Button>
                </div>
            </div>
            <div class="flex w-full items-end gap-2.5">
                <div class="grid gap-2.5 w-full gap-1.5">
                    <Label for="localIp">Local Ip</Label>
                    <Input
                        id="localIp"
                        type="text"
                        placeholder="Local Ip"
                        :value="localIp"
                        readonly
                    />
                </div>
                <div>
                    <Button size="icon">
                        <ClipboardIcon className="h-4 w-4" />
                    </Button>
                </div>
            </div>
        </div>

        <div class="flex flex-col m-auto sm:mt-32 sm:mb-4 w-full">
            <div class="flex py-4 items-center">
                <div
                    class="grid sm:grid-flow-col gap-2 items-center w-full sm:ml-auto sm:w-auto"
                >
                    <Button
                        :class="{ hidden: list.length === 0 }"
                        class="order-2 sm:order-none"
                        variant="destructive"
                        size="lg"
                        :disabled="list.length === 0"
                        @click="clearAllFilesToHost"
                    >
                        <Trash2Icon class="mr-2 h-4 w-4" />
                        Clear All
                    </Button>
                    <Button size="lg" @click="openFilePickerToAddFilesToHost">
                        <PlusSquareIcon class="mr-2 h-4 w-4" />
                        Add files
                    </Button>
                </div>
            </div>

            <FileHostingTable :list="list" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { readDir } from "@tauri-apps/plugin-fs";
import { open, confirm } from "@tauri-apps/plugin-dialog";
import { ref, unref, watch, onBeforeMount, onUnmounted } from "vue";
import { useEventSource } from "@vueuse/core";
import axios from "axios";

import { FileViewModel } from "@/types/File";

import {
    PlusSquareIcon,
    Trash2Icon,
    UploadIcon,
    ClipboardIcon,
} from "lucide-vue-next";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import FileHostingTable from "@/components/organisms/FileHostingTable.vue";

import { toast } from "vue-sonner";

// #region ip
const publicIp = ref<string | undefined>();
const localIp = ref<string | undefined>();
const PORT = 8080;
const getFullAddress = (domain: string) =>
    `http://${domain}${PORT ? `:${PORT}` : ""}`;
onBeforeMount(async () => {
    const { status, code, data } = await axios
        .get("https://api.ipify.org?format=json")
        .catch((error) => error);

    if (!(status >= 200 && status < 300)) {
        toast.error(`Failed to get file list!`, {
            description: `Get file list with status: ${status}; code: ${code}`,
        });

        return;
    }

    publicIp.value = getFullAddress(data.ip);
    localIp.value = getFullAddress(await invoke("get_local_ip"));
});
// #endregion

const { data } = useEventSource(`${getFullAddress("localhost")}/events`);
watch(data, (v) => {
    console.log(v);
});

const unlistenList: Awaited<ReturnType<typeof listen>>[] = [];

// #region listing
const list = ref<FileViewModel[]>([]);
listen("file-added", (event) => {
    list.value = [
        ...unref(list).filter(
            (entry) =>
                entry.id !== event.payload.id &&
                entry.fileName !== event.payload.path,
        ), // tanstack table not deeply reactive: https://github.com/TanStack/table/discussions/4455#discussioncomment-7811125
        {
            id: event.payload.id,
            fileName: event.payload.path,
        },
    ];
}).then((cb) => unlistenList.push(cb));
listen("cleared-all", () => {
    list.value = [];
}).then((cb) => unlistenList.push(cb));
// #endregion

// #region add files to host
const doShowDragAndDrop = ref(false);
const addFilesToHost = (filePaths: string[]) => {
    if (!Array.isArray(filePaths)) throw new Error("No files are selected.");

    filePaths.forEach((filePath) => {
        invoke("add_file", { path: filePath });
    });

    if (filePaths.length === 1) {
        toast.success("Successful Addition!", {
            description: `Your file "${filePaths[0]}" is successfully added.`,
        });
    } else {
        toast.success("Successful Addition!", {
            description: `Your files are successfully added: ${filePaths.join(", ")}`,
        });
    }
};
const openFilePickerToAddFilesToHost = async () => {
    try {
        const files = await open({
            multiple: true,
        });
        if (!files) throw new Error("Failed to select files.");

        addFilesToHost(files.map((file) => file.path));
    } catch (error) {
        toast.error("Failed to add file to host!", {
            // @ts-ignore
            description: error,
        });
    }
};
listen("tauri://file-drop", async (event) => {
    const directoryPaths = (
        await Promise.all(
            event.payload.paths.map(
                async (path) =>
                    await readDir(path)
                        .then(() => path)
                        .catch(() => null),
            ),
        )
    ).filter((v) => v);

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

    try {
        addFilesToHost(event.payload.paths);
    } catch (error) {
        toast.error("Failed to add file to host!", {
            // @ts-ignore
            description: error,
        });
    } finally {
        doShowDragAndDrop.value = false;
    }
}).then((cb) => unlistenList.push(cb));
listen("tauri://file-drop-hover", async () => {
    doShowDragAndDrop.value = true;
}).then((cb) => unlistenList.push(cb));
listen("tauri://file-drop-cancelled", async () => {
    doShowDragAndDrop.value = false;
}).then((cb) => unlistenList.push(cb));
// #endregion

const clearAllFilesToHost = async () => {
    const isConfirmed = await confirm(
        "You're aboout to clear all files to host.",
        {
            title: "Are you sure?",
            type: "warning",
        },
    );

    if (!isConfirmed) return;

    invoke("clear_files");
    toast.success("Successful Clear!", {
        description: `All your files are successfully cleared..`,
    });
};

onUnmounted(() => unlistenList.forEach((unlisten) => unlisten()));
</script>
