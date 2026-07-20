const DotGraph = {
    mounted() {
        this.viz = new Viz();
        this.panZoom = null;

        this.resizeObserver = new ResizeObserver(() => {
            const svg = this.el.querySelector("svg");
            if (svg) this.sizeSvg(svg);
        });
        this.resizeObserver.observe(this.el);

        this.handleEvent("render_dot", ({ dot }) => this.renderDot(dot));
    },

    destroyed() {
        if (this.resizeObserver) this.resizeObserver.disconnect();
        if (this.panZoom) this.panZoom.destroy();
    },

    forceSize(svg) {
        const { width, height } = this.el.getBoundingClientRect();
        svg.setAttribute("width", width);
        svg.setAttribute("height", height);
        svg.style.width = width + "px";
        svg.style.height = height + "px";
    },

    sizeSvg(svg) {
        const { width, height } = this.el.getBoundingClientRect();

        svg.style.position = "absolute";
        svg.style.top = "0";
        svg.style.left = "0";

        svg.setAttribute("width", width);
        svg.setAttribute("height", height);
        svg.style.width = width + "px";
        svg.style.height = height + "px";

        if (this.panZoom) {
            this.panZoom.resize();
            this.panZoom.fit();
            this.panZoom.center();
        }
    },

    async renderDot(dot) {
        if (!dot) return;

        try {
            const svgMarkup = await this.viz.renderString(dot);
            this.el.innerHTML = svgMarkup;

            const svg = this.el.querySelector("svg");
            svg.style.position = "absolute";
            svg.style.top = "0";
            svg.style.left = "0";

            this.forceSize(svg);
            await new Promise(requestAnimationFrame);

            if (this.panZoom) this.panZoom.destroy();

            this.panZoom = svgPanZoom(svg, {
                zoomEnabled: true,
                controlIconsEnabled: true,
                minZoom: 0.1,
                fit: true,
                center: true
            });

            this.forceSize(svg);
        } catch (err) {
            this.el.innerHTML = "<p class='text-red-600 p-3'>Cannot render DOT graph</p>";
            console.error(err);
        }
    }
};

export default DotGraph;