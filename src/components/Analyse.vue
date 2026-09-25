<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
    LogInst,
    NButton,
    NCard,
    NDivider,
    NFlex,
    NLog,
    NScrollbar,
    NSpace,
    NText,
} from 'naive-ui';
import { nextTick, onMounted, onUnmounted, ref } from 'vue';

interface Analyse {
    depth: number;
    score: number;
    time: number;
    pvs: string[];
    moves: string[];
    state: string;
    source: string;
}

interface HistoryEntry {
    seq: number;
    from: string;
    to: string;
    piece: string;
    camp: string;
    iccs: string;
    fen: string;
    source: string;
}

const logs = ref<string[]>([]);
const best = ref({
    move: '等待分析…',
    depth: 0,
    score: 0,
    time: 0,
});

// 高亮组件内状态：只记录当轮 from/to，不全局清所有
const highlight = ref<{ from: string; to: string } | null>(null);

function cellOf(id: string): Element | null {
    return document.getElementById(id)?.firstElementChild ?? document.getElementById(id);
}

function clearHighlight() {
    if (highlight.value) {
        cellOf(highlight.value.from)?.classList.remove('b-select');
        cellOf(highlight.value.to)?.classList.remove('b-select');
    }
    highlight.value = null;
}

function formatTime(date = new Date()) {
    const pad = (n: number) => String(n).padStart(2, '0');
    return `[${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}]`;
}

async function onAnalyse(event: { payload: Analyse }) {
    const data = event.payload;
    const mvs = (data.moves ?? []).join(' ') || '(无招法)';
    logs.value.push(`${formatTime()} <${data.source}> ${mvs}`);
    // 日志上限防内存无限增长（保留最近 128 条）
    if (logs.value.length > 128) logs.value.splice(0, logs.value.length - 128);
    best.value.move = data.moves?.[0] ?? '等待分析…';
    best.value.depth = data.depth ?? 0;
    best.value.score = data.score ?? 0;
    best.value.time = data.time ?? 0;

    // 只清理上一轮的 from/to，再加当前轮
    clearHighlight();
    const pv = data.pvs?.[0];
    if (pv && pv.length >= 4) {
        const from = pv.substring(0, 2);
        const to = pv.substring(2, 4);
        cellOf(from)?.classList.add('b-select');
        cellOf(to)?.classList.add('b-select');
        highlight.value = { from, to };
    }

    await nextTick();
    if (typeof logInstRef.value?.scrollTo === 'function') {
        logInstRef.value.scrollTo({ position: 'bottom', silent: true });
    }
}

// 复盘状态
const history = ref<HistoryEntry[]>([]);
const reviewIndex = ref(0);
const reviewTotal = ref(0);
const reviewFen = ref('');
const exportPath = ref('');

async function clearHistory() {
    try {
        await invoke('clear_history');
        history.value = [];
        reviewIndex.value = 0;
        reviewTotal.value = 0;
        reviewFen.value = '';
    } catch (e) {
        console.error('清空历史失败:', e);
    }
}

async function loadHistory() {
    try {
        history.value = await invoke<HistoryEntry[]>('load_history');
    } catch (e) {
        console.error('加载历史失败:', e);
    }
}

async function reviewStep(index: number) {
    if (index < 0 || index > history.value.length) return;
    try {
        await invoke('review_step', { index });
    } catch (e) {
        console.error('复盘跳步失败:', e);
    }
}

function reviewPrev() {
    if (reviewIndex.value <= 0) return;
    reviewStep(reviewIndex.value - 1);
}

function reviewNext() {
    if (reviewIndex.value >= history.value.length) return;
    reviewStep(reviewIndex.value + 1);
}

function onReviewState(event: { payload: { index: number; total: number; fen: string } }) {
    reviewIndex.value = event.payload.index;
    reviewTotal.value = event.payload.total;
    reviewFen.value = event.payload.fen;
}

async function exportGame(format: 'fen' | 'json') {
    try {
        const path = await invoke<string>('export_game', { format });
        exportPath.value = path;
    } catch (e) {
        console.error('导出失败:', e);
        exportPath.value = '导出失败: ' + String(e);
    }
}

// 注册/解绑监听：避免 HMR/重复挂载累积回调
let unlistenPromise: Promise<() => void> | null = null;
let unlistenReview: (() => void) | null = null;

onUnmounted(async () => {
    const unlisten = await unlistenPromise;
    unlisten?.();
    unlistenReview?.();
});

onMounted(async () => {
    unlistenPromise = listen('analyse', onAnalyse);
    const unreview = await listen('review_state', onReviewState);
    unlistenReview = unreview;
    await loadHistory();
});

const logInstRef = ref<LogInst | null>(null);
</script>

<template>
    <n-card title="局面分析" :bordered="false" class="textlog">
        <n-flex justify="space-between" align="end" wrap>
            <n-text type="info" class="analyse-title" strong data-testid="best-move">
                {{ best.move }}
            </n-text>
            <n-text type="error">{{ best.score }}</n-text>
            <n-text type="warning">{{ best.depth }}</n-text>
            <n-text type="info">{{ best.time }}ms</n-text>
        </n-flex>
        <n-divider />
        <n-log
            class="analyse-log"
            :rows="12"
            ref="logInstRef"
            :line-height="1.5"
            :lines="logs"
            :font-size="12"
            data-testid="analyse-log"
        />

        <n-divider />
        <n-flex justify="space-between" align="center">
            <n-text strong>复盘</n-text>
            <n-space>
                <n-button size="tiny" data-testid="review-prev" :disabled="reviewIndex <= 0" @click="reviewPrev">
                    上一步
                </n-button>
                <n-button
                    size="tiny"
                    data-testid="review-next"
                    :disabled="reviewIndex >= history.length"
                    @click="reviewNext"
                >
                    下一步
                </n-button>
                <n-button size="tiny" data-testid="clear-history" @click="clearHistory">清空</n-button>
            </n-space>
        </n-flex>
        <n-text depth="3" style="font-size: 12px">
            {{ reviewIndex }}/{{ history.length }} 步{{ reviewFen ? ' · ' + reviewFen : '' }}
        </n-text>
        <n-scrollbar style="max-height: 120px; margin-top: 6px">
            <div v-if="history.length === 0" style="color: #8a8a8a; font-size: 12px; padding: 4px">
                暂无对局历史（启动引擎后自动记录）
            </div>
            <div
                v-for="entry in history"
                :key="entry.seq"
                :data-testid="`history-${entry.seq}`"
                style="font-size: 12px; padding: 2px 4px"
            >
                {{ entry.seq }}. {{ entry.piece }} {{ entry.from }}→{{ entry.to }}
                （{{ entry.source }}）
            </div>
        </n-scrollbar>

        <n-divider />
        <n-flex justify="space-between">
            <n-space>
                <n-button size="tiny" data-testid="export-fen" @click="exportGame('fen')">导出 FEN</n-button>
                <n-button size="tiny" data-testid="export-json" @click="exportGame('json')">导出 JSON</n-button>
            </n-space>
        </n-flex>
        <n-text v-if="exportPath" depth="3" style="font-size: 12px; word-break: break-all" data-testid="export-path">
            {{ exportPath }}
        </n-text>
    </n-card>
</template>

<style scoped>
.analyse-title {
    font-size: x-large;
}

.textlog {
    width: 100%;
    min-width: 280px;
    height: auto;
}

.analyse-log {
    font-size: 12px;
}
</style>




