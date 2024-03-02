<template>
    <div class="flex flex-col m-auto sm:mt-32 sm:mb-4">
        <div class="flex py-4 items-center">
            <div class="ml-auto flex gap-2 items-center">
                <Button
                    class="ml-auto"
                    :class="{ hidden: list.length === 0 }"
                    variant="destructive"
                    size="lg"
                    :disabled="list.length === 0"
                    @click="clearAllFilesToHost"
                >
                    <Trash2Icon class="mr-2 h-4 w-4" />
                    Clear All
                </Button>
                <Button size="lg" @click="addFilesToHost">
                    <PlusSquareIcon class="mr-2 h-4 w-4" />
                    Add files
                </Button>
            </div>
        </div>

        <FileHostingTable class="mx-auto w-full" :list="list" />

        <!-- TODO:  tauri drag drop zone -->
    </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { open, confirm } from "@tauri-apps/plugin-dialog";
import { ref } from "vue";

import { FileViewModel } from "@/types/File";

import { PlusSquareIcon, Trash2Icon } from "lucide-vue-next";
import { Button } from "@/components/ui/button";
import FileHostingTable from "@/components/organisms/FileHostingTable.vue";

import { toast } from "vue-sonner";

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

const addFilesToHost = async () => {
    try {
        const files = await open({
            multiple: true,
        });

        if (!files) throw new Error("Failed to select files.");
        if (!Array.isArray(files)) throw new Error("No files are selected.");

        files.forEach((file) => {
            invoke("add_file", { path: file.path });
        });

        if (files.length === 1) {
            toast.success("Successful Addition!", {
                description: `Your file "${files[0].path}" is successfully added.`,
            });
        } else {
            toast.success("Successful Addition!", {
                description: `Your files are successfully added: ${files.map((file) => file.path).join(", ")}`,
            });
        }
    } catch (error) {
        toast.error("Failed to add file to host!", {
            // @ts-ignore
            description: error,
        });
    }
};

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
</script>
