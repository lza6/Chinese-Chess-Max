<script setup lang="ts">
import { emit, listen } from '@tauri-apps/api/event';
import { computed, onMounted, onUnmounted, ref } from 'vue';

import '../assets/css/chessboard.css';

interface Position {
    piece: string;
    pos: string;
}

interface Changed {
    piece: string;
    from: string;
    to: string;
    camp: string;
}

const COLS = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'];

const startpos: Position[] = [
    { piece: 'R', pos: 'a0' }, { piece: 'N', pos: 'b0' }, { piece: 'B', pos: 'c0' },
    { piece: 'A', pos: 'd0' }, { piece: 'K', pos: 'e0' }, { piece: 'A', pos: 'f0' },
    { piece: 'B', pos: 'g0' }, { piece: 'N', pos: 'h0' }, { piece: 'R', pos: 'i0' },
    { piece: 'C', pos: 'b2' }, { piece: 'C', pos: 'h2' },
    { piece: 'P', pos: 'a3' }, { piece: 'P', pos: 'c3' }, { piece: 'P', pos: 'e3' },
    { piece: 'P', pos: 'g3' }, { piece: 'P', pos: 'i3' },
    { piece: 'r', pos: 'a9' }, { piece: 'n', pos: 'b9' }, { piece: 'b', pos: 'c9' },
    { piece: 'a', pos: 'd9' }, { piece: 'k', pos: 'e9' }, { piece: 'a', pos: 'f9' },
    { piece: 'b', pos: 'g9' }, { piece: 'n', pos: 'h9' }, { piece: 'r', pos: 'i9' },
    { piece: 'c', pos: 'b7' }, { piece: 'c', pos: 'h7' },
    { piece: 'p', pos: 'a6' }, { piece: 'p', pos: 'c6' }, { piece: 'p', pos: 'e6' },
    { piece: 'p', pos: 'g6' }, { piece: 'p', pos: 'i6' },
];

// 90 格坐标数据生成（替代硬编码数组）
const mirror = ref(false);

const wrappedItems = computed(() => {
    const ids: string[] = [];
    const colOrder = mirror.value ? [...COLS].reverse() : COLS;
    // 非镜像：从黑方视角底部(a9)往上；镜像：红方视角底部(i0)
    const rowOrder = mirror.value ? [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] : [9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
    for (const r of rowOrder) {
        for (const c of colOrder) {
            ids.push(`${c}${r}`);
        }
    }
    return ids.map((id) => ({ id }));
});

// 全量重绘：先清空所有棋子 class，再按 payload 添加
function setPiecesOnBoard(pieces: Position[]) {
    const wraps = document.querySelectorAll('.piece-wrap');
    wraps.forEach((wrap) => {
        const cell = wrap.firstElementChild;
        if (!cell) return;
        Array.from(cell.classList).forEach((cls) => {
            if (cls.startsWith('piece-') && cls !== 'piece') cell.classList.remove(cls);
        });
    });
    for (const record of pieces) {
        const ele = document.getElementById(record.pos)?.firstElementChild;
        if (!ele) continue;
        if (record.piece !== ' ') ele.classList.add(`piece-${record.piece}`);
    }
}

function onMirror(event: { payload: boolean }) {
    mirror.value = event.payload;
}

function onPosition(event: { payload: Position[] }) {
    setPiecesOnBoard(event.payload);
}

function onMove(event: { payload: Changed }) {
    const change = event.payload;
    const token = `piece-${change.piece}`;

    // 原坐标移除棋子
    document.getElementById(change.from)?.firstElementChild?.classList.remove(token);
    // 目标坐标移除旧棋子（保留空格）
    const ele = document.getElementById(change.to)?.firstElementChild;
    if (ele) {
        Array.from(ele.classList).forEach((cls) => {
            if (cls.startsWith('piece-') && cls !== 'piece') ele.classList.remove(cls);
        });
        ele.classList.add(token);
    }
}

const unlisteners: Array<() => void> = [];
onUnmounted(() => {
    for (const un of unlisteners) un();
});

onMounted(async () => {
    await emit('position', startpos);
    unlisteners.push(await listen('mirror', onMirror));
    unlisteners.push(await listen('position', onPosition));
    unlisteners.push(await listen('move', onMove));
});
</script>

<template>
    <div id="chessboard">
        <div v-for="item in wrappedItems" :key="item.id" :id="item.id" class="piece-wrap">
            <span class="piece"></span>
        </div>
    </div>
</template>

