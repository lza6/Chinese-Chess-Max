<script setup lang="ts">
import { listen } from '@tauri-apps/api/event';
import { LogInst, NCard, NDivider, NFlex, NLog, NText } from 'naive-ui';
import { nextTick, onMounted, onUnmounted, ref } from 'vue';

interface Analyse {
    depth: number,   // 深度
    score: number,   // 得分
    time: number,    // 时间
    pvs: string[],   // 思考(iccs)
    moves: string[], // 思考(chinese)
    state: string,   // 状态
    source: string,  // 来源
}


const logs = ref<string[]>([])
const best = ref({
    move: "----",
    depth: 0,
    score: 0,
    time: 0,
})

async function onAnalyse(event: { payload: Analyse }) {
    const data = event.payload;
    const mvs = (data.moves ?? []).join(" ") || "(无招法)";
    logs.value.push(`<${data.source}> ${mvs}`)
    // 日志上限防内存无限增长（保留最近 128 条）
    if (logs.value.length > 128) logs.value.splice(0, logs.value.length - 128);
    best.value.move = data.moves?.[0] ?? "----";
    best.value.depth = data.depth ?? 0;
    best.value.score = data.score ?? 0;
    best.value.time = data.time ?? 0;

    // 新分析前清理上一轮的高亮
    document.querySelectorAll(".b-select").forEach((el) => el.classList.remove("b-select"));
    // 高亮最佳招法起点/终点（防御空 pv 与无效坐标）
    const pv = data.pvs?.[0];
    if (pv && pv.length >= 4) {
        const from = pv.substring(0, 2);
        const to = pv.substring(2, 4);
        document.getElementById(from)?.classList.add("b-select");
        document.getElementById(to)?.classList.add("b-select");
    }
    // 等 DOM 渲染新行后再滚动到底（jsdom 无 scrollTo，需存在性判断）
    await nextTick();
    if (typeof logInstRef.value?.scrollTo === 'function') {
        logInstRef.value.scrollTo({ position: 'bottom', silent: true });
    }
}

// 注册/解绑监听：避免 HMR/重复挂载累积回调
let unlistenPromise: Promise<() => void> | null = null;
onMounted(() => {
    unlistenPromise = listen('analyse', onAnalyse);
});
onUnmounted(async () => {
    const unlisten = await unlistenPromise;
    unlisten?.();
});


const logInstRef = ref<LogInst | null>(null)

</script>

<template>
    <n-card title="局面分析" :bordered="false" class="textlog" content-style="color: blue">
        <n-flex justify="space-between" align="end">
            <n-text type="info" class="analyse-title" strong>
                {{ best.move }}
            </n-text>

            <n-text type="error">
                {{ best.score }}
            </n-text>
            <n-text type="warning">
                {{ best.depth }}
            </n-text>
            <n-text type="info">
                {{ best.time }}ms
            </n-text>
        </n-flex>
        <n-divider />
        <n-log class="analyse-log" :rows="18" ref="logInstRef" :line-height="1.5" :lines="logs" :font-size="10" />
    </n-card>
</template>

<style scoped>
.analyse-title {
    font-size: x-large;
}

.textlog {
    width: 260px;
    height: 440px;
    left: 400px;
    top: 0px;
}
</style>