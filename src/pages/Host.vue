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
                            class="animate-appear-from-inside m-auto w-full h-full max-w-20"
                        />
                    </div>
                </div>
            </section>
        </Transition>

        <div
            v-if="serverError"
            class="mt-4 p-4 rounded-md border border-destructive bg-destructive/10 text-destructive text-sm"
        >
            {{ serverError }}
        </div>

        <div
            class="sticky top-[env(safe-area-inset-top,0)] py-8 flex space-x-5 bg-background"
        >
            <div class="flex w-full items-end gap-2.5">
                <div class="grid w-full gap-1.5">
                    <Label for="publicIp">Public Ip</Label>
                    <Input
                        id="publicIp"
                        type="text"
                        placeholder="Unavailable"
                        :value="publicIp"
                        readonly
                    />
                </div>
                <div>
                    <Button
                        variant="secondary"
                        size="icon"
                        :disabled="!publicIp"
                        @click="publicIp && copyToClipboard(publicIp)"
                    >
                        <ClipboardIcon class="h-4 w-4" />
                    </Button>
                </div>
            </div>
            <div class="flex w-full items-end gap-2.5">
                <div class="grid w-full gap-1.5">
                    <Label for="localIp">Local Ip</Label>
                    <Input
                        id="localIp"
                        type="text"
                        placeholder="Unavailable"
                        :value="localIp"
                        readonly
                    />
                </div>
                <div>
                    <Button
                        size="icon"
                        :disabled="!localIp"
                        @click="localIp && copyToClipboard(localIp)"
                    >
                        <ClipboardIcon class="h-4 w-4" />
                    </Button>
                </div>
            </div>
        </div>

        <div
            v-if="qrCodeSvg"
            class="flex flex-col items-center gap-2 py-4"
        >
            <!-- eslint-disable-next-line vue/no-v-html — SVG is generated locally from our own URL -->
            <div
                class="w-40 h-40 p-2 rounded-md bg-white [&>svg]:w-full [&>svg]:h-full"
                v-html="qrCodeSvg"
            />
            <p class="text-sm text-muted-foreground">
                Scan to connect from a device on the same network
            </p>
        </div>

        <div class="flex w-full items-end gap-2.5 py-4">
            <div class="grid gap-1.5 w-full">
                <Label for="saveDir">Received files are saved to</Label>
                <Input
                    id="saveDir"
                    type="text"
                    placeholder="Unknown"
                    :value="saveDir"
                    readonly
                />
            </div>
            <div>
                <Button
                    variant="secondary"
                    size="icon"
                    aria-label="Change save folder"
                    @click="changeSaveDir"
                >
                    <FolderIcon class="h-4 w-4" />
                </Button>
            </div>
        </div>

        <section v-if="receivedFiles.length > 0" class="py-4">
            <h2 class="text-lg font-semibold pb-2">Received files</h2>
            <ul class="rounded-md border divide-y">
                <li
                    v-for="file in receivedFiles"
                    :key="file.path"
                    class="flex items-center gap-2.5 p-3 text-sm"
                >
                    <span class="truncate">{{ file.name }}</span>
                    <span class="ml-auto whitespace-nowrap text-muted-foreground">
                        {{ formatBytes(file.size) }}
                    </span>
                    <Button
                        variant="ghost"
                        size="icon"
                        :aria-label="`Reveal ${file.name} in folder`"
                        @click="revealInFolder(file.path)"
                    >
                        <FolderOpenIcon class="h-4 w-4" />
                    </Button>
                </li>
            </ul>
        </section>

        <div class="flex flex-col m-auto sm:mt-16 sm:mb-4 w-full">
            <div class="flex py-4 items-center">
                <div
                    class="grid sm:grid-flow-col gap-2 items-center w-full sm:ml-auto sm:w-auto"
                >
                    <Button
                        :class="{ hidden: list.length === 0 }"
                        class="order-2 sm:order-0"
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

            <FileHostingTable :list="list" @remove="removeFileFromHost" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen, TauriEvent } from "@tauri-apps/api/event";
import { readDir } from "@tauri-apps/plugin-fs";
import { open, confirm } from "@tauri-apps/plugin-dialog";
import { ref, unref, computed, onBeforeMount, onUnmounted } from "vue";
import { renderSVG } from "uqr";

import { FileViewModel, ReceivedFileViewModel } from "@/types/File";

import {
    PlusSquareIcon,
    Trash2Icon,
    UploadIcon,
    ClipboardIcon,
    FolderIcon,
    FolderOpenIcon,
} from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import FileHostingTable from "@/components/organisms/FileHostingTable.vue";
import { formatBytes } from "@/lib/utils";

import { toast } from "vue-sonner";

const unlistenList: Awaited<ReturnType<typeof listen>>[] = [];

const copyToClipboard = async (text: string) => {
    try {
        await navigator.clipboard.writeText(text);
        toast.success("Successful Copy!", {
            description: `"${text}" is successfully copied to clipboard.`,
        });
    } catch {
        toast.error("Failed to copy to clipboard!");
    }
};

// #region ip
const publicIp = ref<string | undefined>();
const localIp = ref<string | undefined>();
const PORT = 8080;
const getFullAddress = (domain: string) =>
    `http://${domain}${PORT ? `:${PORT}` : ""}`;
onBeforeMount(async () => {
    // resolved independently: no network shouldn't blank out the local address
    invoke<string>("get_local_ip")
        .then((ip) => (localIp.value = getFullAddress(ip)))
        .catch(() => {});
    invoke<string>("get_public_ip")
        .then((ip) => (publicIp.value = getFullAddress(ip)))
        .catch(() => {});
});

const qrCodeSvg = computed(() => {
    const url = unref(localIp);
    return url ? renderSVG(url, { pixelSize: 4 }) : undefined;
});
// #endregion

// #region server status
const serverError = ref<string | undefined>();
onBeforeMount(async () => {
    serverError.value =
        (await invoke<string | null>("get_server_error")) ?? undefined;
});
listen<string>("server-error", (event) => {
    serverError.value = event.payload;
}).then((cb) => unlistenList.push(cb));
// #endregion

// #region save directory
const saveDir = ref<string | undefined>();
onBeforeMount(async () => {
    saveDir.value = await invoke<string>("get_save_dir");
});
const changeSaveDir = async () => {
    const directory = await open({ directory: true });
    if (!directory) return;

    try {
        saveDir.value = await invoke<string>("set_save_dir", {
            path: directory,
        });
        toast.success("Save folder updated!", {
            description: `Received files will be saved to "${directory}".`,
        });
    } catch (error) {
        toast.error("Failed to change save folder!", {
            description: String(error),
        });
    }
};
// #endregion

// #region listing
const list = ref<FileViewModel[]>([]);
listen<{ id: string; path: string; size: number }>("file-added", (event) => {
    list.value = [
        ...unref(list).filter(
            (entry) =>
                entry.id !== event.payload.id &&
                entry.fileName !== event.payload.path,
        ), // tanstack table not deeply reactive: https://github.com/TanStack/table/discussions/4455#discussioncomment-7811125
        {
            id: event.payload.id,
            fileName: event.payload.path,
            size: event.payload.size,
        },
    ];
}).then((cb) => unlistenList.push(cb));
listen<string>("file-removed", (event) => {
    list.value = unref(list).filter((entry) => entry.id !== event.payload);
}).then((cb) => unlistenList.push(cb));
listen("cleared-all", () => {
    list.value = [];
}).then((cb) => unlistenList.push(cb));
// #endregion

// #region received files
const receivedFiles = ref<ReceivedFileViewModel[]>([]);
listen<ReceivedFileViewModel>("file-received", (event) => {
    receivedFiles.value = [
        event.payload,
        ...unref(receivedFiles).filter(
            (file) => file.path !== event.payload.path,
        ),
    ];
    toast.info("File received!", {
        description: `"${event.payload.name}" was saved to your save folder.`,
    });
}).then((cb) => unlistenList.push(cb));
const revealInFolder = async (path: string) => {
    try {
        await invoke("reveal_in_folder", { path });
    } catch (error) {
        toast.error("Failed to open folder!", {
            description: String(error),
        });
    }
};
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

        addFilesToHost(files);
    } catch (error) {
        toast.error("Failed to add file to host!", {
            description: String(error),
        });
    }
};
listen<{ paths: string[] }>(TauriEvent.DRAG_DROP, async (event) => {
    const directoryPaths = (
        await Promise.all(
            event.payload.paths.map(
                async (path) =>
                    await readDir(path)
                        .then(() => path)
                        .catch(() => null),
            ),
        )
    ).filter((v): v is string => !!v);

    if (directoryPaths.length > 0) {
        toast.error(
            directoryPaths.length === 1
                ? "Cannot add directory!"
                : "Cannot add directories!",
            {
                description: directoryPaths.join(", "),
            },
        );
        doShowDragAndDrop.value = false;
        return;
    }

    try {
        addFilesToHost(event.payload.paths);
    } catch (error) {
        toast.error("Failed to add file to host!", {
            description: String(error),
        });
    } finally {
        doShowDragAndDrop.value = false;
    }
}).then((cb) => unlistenList.push(cb));
listen(TauriEvent.DRAG_ENTER, async () => {
    doShowDragAndDrop.value = true;
}).then((cb) => unlistenList.push(cb));
listen(TauriEvent.DRAG_LEAVE, async () => {
    doShowDragAndDrop.value = false;
}).then((cb) => unlistenList.push(cb));
// #endregion

const removeFileFromHost = (id: string) => {
    invoke("remove_file", { id });
};

const clearAllFilesToHost = async () => {
    const isConfirmed = await confirm(
        "You're about to clear all files to host.",
        {
            title: "Are you sure?",
            kind: "warning",
        },
    );

    if (!isConfirmed) return;

    invoke("clear_files");
    toast.success("Successful Clear!", {
        description: `All your files are successfully cleared.`,
    });
};

onUnmounted(() => unlistenList.forEach((unlisten) => unlisten()));
</script>
