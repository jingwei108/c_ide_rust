// C IDE CodeMirror 6 Interop — adds breakpoint, error, and execution line decorations
// Requires GaelJ.BlazorCodeMirror6 to be loaded first.

(function () {
    'use strict';

    let gaeljModule = null;
    let gaeljModulePromise = null;

    async function ensureModule() {
        if (gaeljModule) return gaeljModule;
        if (gaeljModulePromise) return await gaeljModulePromise;
        // Use absolute URL to work around WebView file:// protocol ES module resolution
        var moduleUrl = new URL('_content/GaelJ.BlazorCodeMirror6/index.js', document.baseURI).href;
        gaeljModulePromise = import(moduleUrl);
        gaeljModule = await gaeljModulePromise;
        return gaeljModule;
    }

    function getView(id) {
        const cm = gaeljModule?.getCmInstance(id);
        return cm ? cm.view : null;
    }

    // Find the line number gutter container
    function getLineNumberGutter(view) {
        return view?.dom?.querySelector('.cm-gutters .cm-lineNumbers');
    }

    // Map line numbers to their DOM elements (gutter + content)
    function getLineElements(view) {
        const gutter = getLineNumberGutter(view);
        if (!gutter) return [];

        const gutterEls = Array.from(gutter.querySelectorAll('.cm-gutterElement'));
        const contentEls = Array.from(view.dom.querySelectorAll('.cm-content .cm-line'));

        const result = [];
        // Gutter and content elements are in 1:1 order for visible lines
        for (let i = 0; i < gutterEls.length && i < contentEls.length; i++) {
            const text = gutterEls[i].textContent?.trim();
            const lineNum = parseInt(text, 10);
            if (!isNaN(lineNum)) {
                result.push({ line: lineNum, gutter: gutterEls[i], content: contentEls[i] });
            }
        }
        return result;
    }

    // State
    const breakpoints = new Map(); // id -> Set(line)
    const errorLines = new Map();  // id -> Set(line)
    const highlightLine = new Map(); // id -> line

    function applyDecorations(id) {
        const view = getView(id);
        if (!view) return;

        const bpSet = breakpoints.get(id) || new Set();
        const errSet = errorLines.get(id) || new Set();
        const hlLine = highlightLine.get(id) ?? -1;

        const lines = getLineElements(view);
        for (const item of lines) {
            const { line, gutter, content } = item;

            // Breakpoint: add/remove red dot in gutter
            let bpDot = gutter.querySelector('.cide-bp-dot');
            if (bpSet.has(line)) {
                if (!bpDot) {
                    bpDot = document.createElement('span');
                    bpDot.className = 'cide-bp-dot';
                    bpDot.style.cssText = 'position:absolute;left:2px;top:50%;transform:translateY(-50%);width:8px;height:8px;border-radius:50%;background:#ff4444;box-shadow:0 0 2px rgba(255,68,68,0.6);';
                    gutter.style.position = 'relative';
                    gutter.appendChild(bpDot);
                }
            } else if (bpDot) {
                bpDot.remove();
            }

            // Error line: red background on content
            content.classList.toggle('cide-error-line', errSet.has(line));

            // Highlight line: yellow/green background on content
            content.classList.toggle('cide-highlight-line', hlLine === line);
        }
    }

    // Re-apply decorations on scroll / resize so virtual scrolling gets updated
    const observerMap = new Map();
    function ensureObserver(id) {
        if (observerMap.has(id)) return;
        const view = getView(id);
        if (!view) return;

        const dom = view.dom;
        const onChange = () => applyDecorations(id);

        dom.addEventListener('scroll', onChange, { passive: true });
        window.addEventListener('resize', () => {
            const v = getView(id);
            if (v) v.requestMeasure();
            onChange();
        });

        const mutObs = new MutationObserver(onChange);
        mutObs.observe(dom.querySelector('.cm-content') || dom, { childList: true, subtree: true });

        observerMap.set(id, {
            disconnect() {
                dom.removeEventListener('scroll', onChange);
                window.removeEventListener('resize', onChange);
                mutObs.disconnect();
            }
        });
    }

    // Public API
    window.cideCodeMirrorInterop = {
        async setBreakpoints(id, lines) {
            await ensureModule();
            breakpoints.set(id, new Set(lines));
            applyDecorations(id);
            ensureObserver(id);
        },

        async setErrorLines(id, lines) {
            await ensureModule();
            errorLines.set(id, new Set(lines));
            applyDecorations(id);
            ensureObserver(id);
        },

        async setHighlightLine(id, line) {
            await ensureModule();
            highlightLine.set(id, line);
            applyDecorations(id);
            ensureObserver(id);
        },

        async insertTemplate(id, text) {
            await ensureModule();
            const view = getView(id);
            if (!view || !text) return;
            const pos = view.state.selection.main.head;
            view.dispatch({
                changes: { from: pos, to: pos, insert: text }
            });
        },

        async destroy(id) {
            const obs = observerMap.get(id);
            if (obs) {
                obs.disconnect();
                observerMap.delete(id);
            }
            breakpoints.delete(id);
            errorLines.delete(id);
            highlightLine.delete(id);
        }
    };
})();
