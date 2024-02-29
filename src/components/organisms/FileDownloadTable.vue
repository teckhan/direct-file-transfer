<template>
    <div>
        <div class="flex py-4" items-center>
            <div class="ml-auto flex gap-2 items-center">
                <Button variant="secondary" size="lg" @click="$emit('upload')">
                    <UploadIcon class="mr-2 h-4 w-4" />
                    Upload
                </Button>
                <Button
                    class="ml-auto"
                    size="lg"
                    :disabled="list?.length === 0"
                    @click="$emit('download-all')"
                >
                    <DownloadIcon class="mr-2 h-4 w-4" />
                    Download All
                </Button>
            </div>
        </div>
        <div class="rounded-md border">
            <Table>
                <TableHeader>
                    <TableRow
                        v-for="headerGroup in table.getHeaderGroups()"
                        :key="headerGroup.id"
                    >
                        <TableHead
                            v-for="header in headerGroup.headers"
                            :key="header.id"
                            :data-pinned="header.column.getIsPinned()"
                            :class="
                                cn(
                                    {
                                        'sticky bg-background/95':
                                            header.column.getIsPinned(),
                                    },
                                    header.column.getIsPinned() === 'left'
                                        ? 'left-0'
                                        : 'right-0',
                                )
                            "
                        >
                            <FlexRender
                                v-if="!header.isPlaceholder"
                                :render="header.column.columnDef.header"
                                :props="header.getContext()"
                            />
                        </TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    <template v-if="table.getRowModel().rows?.length">
                        <TableRow
                            v-for="row in table.getRowModel().rows"
                            :key="row.id"
                        >
                            <TableCell
                                v-for="cell in row.getVisibleCells()"
                                :key="cell.id"
                                :data-pinned="cell.column.getIsPinned()"
                                :class="
                                    cn(
                                        {
                                            'sticky bg-background/95':
                                                cell.column.getIsPinned(),
                                        },
                                        cell.column.getIsPinned() === 'left'
                                            ? 'left-0'
                                            : 'right-0',
                                        {
                                            'w-[1%] whitespace-nowrap':
                                                cell.column.id === 'actions',
                                        },
                                    )
                                "
                            >
                                <FlexRender
                                    :render="cell.column.columnDef.cell"
                                    :props="cell.getContext()"
                                />
                            </TableCell>
                        </TableRow>
                    </template>

                    <TableRow v-else>
                        <TableCell
                            col-span="{columns.length}"
                            class="h-24 text-center w-full"
                        >
                            No results.
                        </TableCell>
                    </TableRow>
                </TableBody>
            </Table>
        </div>

        <div class="flex items-center justify-end space-x-2 py-4">
            <div class="space-x-2">
                <Button
                    variant="outline"
                    size="sm"
                    :disabled="!table.getCanPreviousPage()"
                    @click="table.previousPage()"
                >
                    Previous
                </Button>
                <Button
                    variant="outline"
                    size="sm"
                    :disabled="!table.getCanNextPage()"
                    @click="table.nextPage()"
                >
                    Next
                </Button>
            </div>
        </div>
    </div>
</template>

<script lang="ts" setup>
import { h } from "vue";

import { FileViewModel } from "@/types/File";

import {
    FlexRender,
    createColumnHelper,
    getCoreRowModel,
    getPaginationRowModel,
    useVueTable,
} from "@tanstack/vue-table";
import { DownloadIcon, UploadIcon } from "lucide-vue-next";

import { Button } from "@/components/ui/button";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import { cn } from "@/lib/utils";

const props = withDefaults(
    defineProps<{
        list: FileViewModel[];
    }>(),
    {
        list: () => [],
    },
);

const emit = defineEmits<{
    download: [id: string];
    "download-all": [];
    upload: [];
}>();

const columnHelper = createColumnHelper<FileViewModel>();
const columns = [
    columnHelper.accessor("fileName", {
        header: () => h("div", null, "File Name"),
        cell: ({ row }) =>
            h("div", { class: "lowercase" }, row.getValue("fileName")),
    }),
    columnHelper.display({
        id: "actions",
        enableHiding: false,
        header: () => h("div", { class: "text-right" }, "Actions"),
        cell: ({ row }) => {
            return h(
                Button,
                {
                    variant: "secondary",
                    onClick: () => emit("download", row.original.id),
                },
                () => [h(DownloadIcon, { class: "mr-2 h-4 w-4" }), "Download"],
            );
        },
    }),
];

const table = useVueTable({
    get data() {
        return props.list;
    },
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
});
</script>
