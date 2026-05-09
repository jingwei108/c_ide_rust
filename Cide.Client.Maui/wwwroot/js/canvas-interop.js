window.canvasInterop = {
    drawLinkedList: function (canvasId, nodes) {
        const canvas = document.getElementById(canvasId);
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        const dpr = window.devicePixelRatio || 1;
        const rect = canvas.getBoundingClientRect();
        canvas.width = rect.width * dpr;
        canvas.height = rect.height * dpr;
        ctx.scale(dpr, dpr);

        ctx.clearRect(0, 0, rect.width, rect.height);
        if (!nodes || nodes.length === 0) return;

        const nodeW = 70, nodeH = 40;
        const nodeMap = {};
        nodes.forEach(n => nodeMap[n.address] = n);

        // Draw edges first
        ctx.strokeStyle = '#808080';
        ctx.lineWidth = 1.5;
        nodes.forEach(n => {
            if (n.nextAddr && nodeMap[n.nextAddr]) {
                const to = nodeMap[n.nextAddr];
                const x1 = n.x + nodeW / 2;
                const y1 = n.y + nodeH;
                const x2 = to.x + nodeW / 2;
                const y2 = to.y;
                ctx.beginPath();
                ctx.moveTo(x1, y1);
                ctx.lineTo(x2, y2);
                ctx.stroke();

                // Arrowhead
                const angle = Math.atan2(y2 - y1, x2 - x1);
                ctx.beginPath();
                ctx.moveTo(x2, y2);
                ctx.lineTo(x2 - 8 * Math.cos(angle - 0.5), y2 - 8 * Math.sin(angle - 0.5));
                ctx.lineTo(x2 - 8 * Math.cos(angle + 0.5), y2 - 8 * Math.sin(angle + 0.5));
                ctx.fillStyle = '#808080';
                ctx.fill();
            }
        });

        // Draw nodes
        nodes.forEach(n => {
            const x = n.x, y = n.y;
            const bg = n.flashColor || (n.isHighlighted ? '#0A84FF' : '#3c3c3c');
            const border = n.flashColor ? '#FFFFFF' : (n.isHighlighted ? '#4FC1FF' : '#555');
            ctx.fillStyle = bg;
            ctx.strokeStyle = border;
            ctx.lineWidth = n.flashColor ? 3 : 2;
            ctx.beginPath();
            ctx.roundRect(x, y, nodeW, nodeH, 6);
            ctx.fill();
            ctx.stroke();

            ctx.fillStyle = '#fff';
            ctx.font = '13px sans-serif';
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillText(n.label, x + nodeW / 2, y + nodeH / 2);
        });
    },

    drawMemoryMap: function (canvasId, regions) {
        const canvas = document.getElementById(canvasId);
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        const dpr = window.devicePixelRatio || 1;
        const rect = canvas.getBoundingClientRect();
        canvas.width = rect.width * dpr;
        canvas.height = rect.height * dpr;
        ctx.scale(dpr, dpr);

        ctx.clearRect(0, 0, rect.width, rect.height);
        if (!regions || regions.length === 0) return;

        const cellW = 70, cellH = 50, gap = 8;
        const cols = Math.floor(rect.width / (cellW + gap));

        regions.forEach((r, i) => {
            const col = i % cols;
            const row = Math.floor(i / cols);
            const x = 10 + col * (cellW + gap);
            const y = 10 + row * (cellH + gap);

            // Background
            ctx.fillStyle = r.isFreed ? '#555' : r.isHeap ? 'rgba(255,193,7,0.2)' : 'rgba(10,132,255,0.2)';
            ctx.strokeStyle = r.isFreed ? '#777' : r.isHeap ? '#ffc107' : '#0A84FF';
            ctx.lineWidth = 1.5;
            ctx.beginPath();
            ctx.roundRect(x, y, cellW, cellH, 4);
            ctx.fill();
            ctx.stroke();

            // Text
            ctx.fillStyle = r.isFreed ? '#888' : '#d4d4d4';
            ctx.font = '10px sans-serif';
            ctx.textAlign = 'center';
            ctx.textBaseline = 'top';
            ctx.fillText(r.name, x + cellW / 2, y + 4);
            ctx.fillText(`0x${r.address.toString(16).toUpperCase()}`, x + cellW / 2, y + 16);
            ctx.fillText(`${r.value}`, x + cellW / 2, y + 28);
            ctx.fillText(`${r.size}b`, x + cellW / 2, y + 40);
        });
    }
};
