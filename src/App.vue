<script setup lang="ts">
import { NDialogProvider, NLayoutFooter, NMessageProvider } from "naive-ui";
import Analyse from "./components/Analyse.vue";
import Chessboard from "./components/Chessboard.vue";
import Toolbar from "./components/Toolbar.vue";
</script>

<template>
    <n-dialog-provider>
        <n-message-provider>
            <div class="app-layout">
                <header class="app-header">
                    <Toolbar />
                </header>

                <main class="app-main">
                    <section class="board-area">
                        <Chessboard />
                    </section>
                    <aside class="analyse-area">
                        <Analyse />
                    </aside>
                </main>

                <n-layout-footer class="footer">
                    <a href="https://github.com/lza6/Chinese-Chess-Max"
                        >Chinese-Chess-Max · 开源免费</a
                    >
                </n-layout-footer>
            </div>
        </n-message-provider>
    </n-dialog-provider>
</template>

<style scoped>
.app-layout {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    gap: 12px;
    padding: 12px;
    box-sizing: border-box;
}

.app-header {
    flex: 0 0 auto;
}

.app-main {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    flex: 1;
    align-items: flex-start;
}

.board-area {
    flex: 1 1 380px;
    min-width: 0;
}

.analyse-area {
    flex: 1 1 300px;
    min-width: 0;
    max-width: 520px;
}

.footer {
    font-size: 12px;
    text-align: center;
    padding: 8px 0;
    position: static;
}

/* 断点 2：中屏（600-1023px）上下堆叠，不横向滚动 */
@media (max-width: 1023px) {
    .app-main {
        flex-direction: column;
        align-items: center;
    }
    .board-area,
    .analyse-area {
        max-width: 100%;
        width: 100%;
    }
}

/* 断点 3：小屏（<600px）紧凑边距，棋盘收缩 */
@media (max-width: 600px) {
    .app-layout {
        padding: 6px;
        gap: 8px;
    }
    .footer {
        font-size: 10px;
    }
}
</style>
