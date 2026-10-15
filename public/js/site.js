(() => {
    const state = { lenis: null, typed: false };

    const prefersReducedMotion = () => matchMedia('(prefers-reduced-motion: reduce)').matches;

    function smoothScroll() {
        if (state.lenis || !window.Lenis || prefersReducedMotion()) return;

        const main = document.querySelector('main');
        const lenis = new Lenis({
            wrapper: main,
            content: main,
            eventsTarget: window,
            lerp: 0.1,
            __experimental__naiveDimensions: true,
        });
        const frame = (time) => {
            lenis.raf(time);
            requestAnimationFrame(frame);
        };

        state.lenis = lenis;
        requestAnimationFrame(frame);
    }

    function resetScroll() {
        if (state.lenis) state.lenis.scrollTo(0, { immediate: true });
        else document.querySelector('main')?.scrollTo(0, 0);
    }

    function reveal(selector) {
        const observer = new IntersectionObserver(
            (entries) => {
                entries
                    .filter((entry) => entry.isIntersecting)
                    .forEach((entry, index) => {
                        entry.target.style.transitionDelay = `${index * 120}ms`;
                        entry.target.classList.add('visible');
                        observer.unobserve(entry.target);
                    });
            },
            { threshold: 0.15 },
        );

        document.querySelectorAll(selector).forEach((element) => observer.observe(element));
    }

    function textNodes(root) {
        const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
        const nodes = [];

        while (walker.nextNode()) nodes.push(walker.currentNode);

        return nodes;
    }

    // The text stays laid out but transparent, and a CSS highlight paints the
    // typed part, so the layout never changes while typing.
    function typewrite() {
        const root = document.querySelector('article');
        if (!root) return;

        const supported = window.CSS && CSS.highlights;
        if (state.typed || !supported || prefersReducedMotion()) {
            root.classList.remove('typing');
            return;
        }

        state.typed = true;

        const nodes = textNodes(root);
        const ends = [];
        let total = 0;

        for (const node of nodes) {
            total += node.data.length;
            ends.push(total);
        }

        const caret = document.createElement('span');
        caret.className = 'caret';
        root.appendChild(caret);

        const highlight = new Highlight();
        CSS.highlights.set('typed', highlight);

        const finish = () => {
            root.classList.remove('typing');
            CSS.highlights.delete('typed');
            setTimeout(() => caret.remove(), 1200);
        };

        let typed = 0;

        const tick = () => {
            if (typed >= total) return finish();

            try {
                typed += 1;

                let index = 0;
                while (ends[index] < typed) index += 1;

                const offset = typed - (index ? ends[index - 1] : 0);
                const range = new Range();
                range.setStart(nodes[0], 0);
                range.setEnd(nodes[index], offset);
                highlight.clear();
                highlight.add(range);

                const last = new Range();
                last.setStart(nodes[index], offset - 1);
                last.setEnd(nodes[index], offset);

                const rect = last.getBoundingClientRect();
                const base = root.getBoundingClientRect();
                caret.style.left = `${rect.right - base.left}px`;
                caret.style.top = `${rect.top - base.top}px`;
                caret.style.height = `${rect.height}px`;

                const pause = /[.,:]/.test(nodes[index].data[offset - 1]);
                setTimeout(tick, pause ? 60 : 7);
            } catch (error) {
                console.error(error);
                finish();
            }
        };

        setTimeout(tick, 200);
    }

    window.site = { smoothScroll, resetScroll, reveal, typewrite };
})();
