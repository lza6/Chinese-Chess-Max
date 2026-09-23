declare global {
    interface Window {
        matchMedia: (query: string) => MediaQueryList;
        ResizeObserver: any;
        requestAnimationFrame: any;
        cancelAnimationFrame: any;
    }
}

// jsdom 缺失但在 naive-ui 渲染中需要的浏览器 API
if (!window.matchMedia) {
    window.matchMedia = ((query: string) => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: () => {},
        removeListener: () => {},
        addEventListener: () => {},
        removeEventListener: () => {},
        dispatchEvent: () => false,
    })) as unknown as typeof window.matchMedia;
}

if (!("ResizeObserver" in window)) {
    (window as any).ResizeObserver = class {
        observe() {}
        unobserve() {}
        disconnect() {}
    };
}

if (!window.requestAnimationFrame) {
    window.requestAnimationFrame = (cb: FrameRequestCallback) =>
        setTimeout(() => cb(Date.now()), 16) as unknown as number;
}
if (!window.cancelAnimationFrame) {
    window.cancelAnimationFrame = (id: number) => clearTimeout(id);
}
