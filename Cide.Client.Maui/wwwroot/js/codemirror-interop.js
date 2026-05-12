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

    // VS-style single-line auto formatter
    function formatLine(text) {
        const leadingMatch = text.match(/^(\s*)(.*)$/);
        if (!leadingMatch) return text;
        const prefix = leadingMatch[1];
        let s = leadingMatch[2];
        if (!s) return text;

        let result = '';
        let i = 0;
        let lastChar = '';
        let inString = false;
        let stringChar = '';

        while (i < s.length) {
            const ch = s[i];
            const prev = s[i - 1] || '';

            // Strings
            if ((ch === '"' || ch === "'") && prev !== '\\') {
                if (!inString) {
                    inString = true;
                    stringChar = ch;
                    if (lastChar && /[a-zA-Z0-9_)]/.test(lastChar)) {
                        result += ' ';
                        lastChar = ' ';
                    }
                    result += ch;
                    lastChar = ch;
                    i++;
                    continue;
                } else if (stringChar === ch) {
                    inString = false;
                    result += ch;
                    lastChar = ch;
                    i++;
                    continue;
                }
            }
            if (inString) {
                result += ch;
                lastChar = ch;
                i++;
                continue;
            }

            // Comments: preserve rest of line
            if (ch === '/' && (s[i + 1] === '/' || s[i + 1] === '*')) {
                if (lastChar && lastChar !== ' ') {
                    result += ' ';
                    lastChar = ' ';
                }
                result += s.slice(i);
                break;
            }

            // Collapse whitespace
            if (/\s/.test(ch)) {
                if (lastChar && !/\s/.test(lastChar)) {
                    result += ' ';
                    lastChar = ' ';
                }
                i++;
                continue;
            }

            // Comma
            if (ch === ',') {
                result += ', ';
                lastChar = ' ';
                i++;
                continue;
            }

            // Semicolon: no space before
            if (ch === ';') {
                if (lastChar === ' ') {
                    result = result.slice(0, -1);
                }
                result += ';';
                lastChar = ';';
                i++;
                continue;
            }

            // Parentheses and brackets
            if (ch === '(') {
                if (lastChar && /[a-zA-Z0-9_]/.test(lastChar)) {
                    result += ' ';
                }
                result += '(';
                lastChar = '(';
                i++;
                continue;
            }
            if (ch === ')') {
                if (lastChar === ' ') {
                    result = result.slice(0, -1);
                }
                result += ')';
                lastChar = ')';
                i++;
                continue;
            }
            if (ch === '[') {
                if (lastChar === ' ') {
                    result = result.slice(0, -1);
                }
                result += '[';
                lastChar = '[';
                i++;
                continue;
            }
            if (ch === ']') {
                if (lastChar === ' ') {
                    result = result.slice(0, -1);
                }
                result += ']';
                lastChar = ']';
                i++;
                continue;
            }
            if (ch === '{') {
                if (lastChar && /[a-zA-Z0-9_)]/.test(lastChar)) {
                    result += ' ';
                }
                result += '{';
                lastChar = '{';
                i++;
                continue;
            }
            if (ch === '}') {
                if (lastChar === ' ') {
                    result = result.slice(0, -1);
                }
                result += '}';
                lastChar = '}';
                i++;
                continue;
            }

            // Two-char operators
            const twoChar = ch + (s[i + 1] || '');
            const twoOps = ['==', '!=', '<=', '>=', '&&', '||', '<<', '>>', '+=', '-=', '*=', '/=', '%=', '&=', '|=', '^=', '->'];
            if (twoOps.includes(twoChar)) {
                if (lastChar && /[a-zA-Z0-9_\)\]]/.test(lastChar) && lastChar !== ' ') {
                    result += ' ';
                }
                result += twoChar;
                lastChar = twoChar[1];
                i += 2;
                continue;
            }

            // Single-char operators
            const singleOps = ['=', '<', '>', '+', '-', '*', '/', '%', '&', '|', '^', '?', ':', '!', '~'];
            if (singleOps.includes(ch)) {
                const isUnaryLike = !lastChar || /[\s\(\[\{,;=\+\-\*\/%&\|\^<>!~]/.test(lastChar);
                if (!isUnaryLike && lastChar !== ' ') {
                    result += ' ';
                }
                result += ch;
                lastChar = ch;
                i++;
                continue;
            }

            // Default identifier/number chars
            result += ch;
            lastChar = ch;
            i++;
        }

        return prefix + result;
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

        // Auto-format previous line on Enter (VS-style)
        dom.addEventListener('keydown', (e) => {
            if (e.key !== 'Enter') return;
            setTimeout(() => {
                const v = getView(id);
                if (!v) return;
                const state = v.state;
                const pos = state.selection.main.head;
                const cur = state.doc.lineAt(pos);
                if (cur.number > 1) {
                    const prev = state.doc.line(cur.number - 1);
                    const formatted = formatLine(prev.text);
                    if (formatted !== prev.text) {
                        v.dispatch({
                            changes: { from: prev.from, to: prev.to, insert: formatted }
                        });
                    }
                }
            }, 0);
        });

        // Symbol bar visibility: show when editor is focused, hide when blurred
        const symbolBar = document.getElementById('symbol-bar');
        const cmContent = dom.querySelector('.cm-content');
        if (symbolBar && cmContent) {
            const showBar = () => symbolBar.classList.add('visible');
            const hideBar = () => {
                setTimeout(() => {
                    if (!cmContent.matches(':focus')) {
                        symbolBar.classList.remove('visible');
                    }
                }, 150);
            };
            cmContent.addEventListener('focus', showBar);
            cmContent.addEventListener('blur', hideBar);
        }

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

        async scrollToLine(id, line) {
            await ensureModule();
            const view = getView(id);
            if (!view) return;
            const doc = view.state.doc;
            if (line < 1 || line > doc.lines) return;
            const lineObj = doc.line(line);
            // Set selection to the start of the line
            view.dispatch({ selection: { anchor: lineObj.from } });
            // Scroll the line into the center of the viewport
            const block = view.lineBlockAt(lineObj.from);
            if (block) {
                view.scrollDOM.scrollTop = block.top - view.scrollDOM.clientHeight / 2 + block.height / 2;
            }
            view.focus();
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

        async insertPair(id, open, close) {
            await ensureModule();
            const view = getView(id);
            if (!view) return;
            const pos = view.state.selection.main.head;
            view.dispatch({
                changes: { from: pos, to: pos, insert: open + close },
                selection: { anchor: pos + open.length }
            });
        },

        async moveCursor(id, offset) {
            await ensureModule();
            const view = getView(id);
            if (!view) return;
            const pos = view.state.selection.main.head;
            const newPos = Math.max(0, Math.min(pos + offset, view.state.doc.length));
            view.dispatch({ selection: { anchor: newPos } });
            view.focus();
        },

        async undo(id) {
            await ensureModule();
            const mod = gaeljModule;
            if (mod?.dispatchCommand) {
                mod.dispatchCommand(id, 'Undo');
            }
        },

        async redo(id) {
            await ensureModule();
            const mod = gaeljModule;
            if (mod?.dispatchCommand) {
                mod.dispatchCommand(id, 'Redo');
            }
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
