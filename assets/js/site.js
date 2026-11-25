(() => {
    const state = { lenis: null, main: null, looping: false, typed: false, typing: false };

    const prefersReducedMotion = () => matchMedia('(prefers-reduced-motion: reduce)').matches;

    function smoothScroll() {
        const main = document.querySelector('main');
        if (!main || !window.Lenis || prefersReducedMotion()) return;
        if (state.lenis && state.main === main) return;

        if (state.lenis) state.lenis.destroy();

        state.main = main;
        state.lenis = new Lenis({
            wrapper: main,
            content: main,
            eventsTarget: window,
            lerp: 0.1,
            __experimental__naiveDimensions: true,
        });

        if (state.looping) return;

        state.looping = true;

        const frame = (time) => {
            if (state.lenis) state.lenis.raf(time);
            requestAnimationFrame(frame);
        };

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
                        entry.target.dataset.visible = '';
                        observer.unobserve(entry.target);
                    });
            },
            { threshold: 0.15 },
        );

        document.querySelectorAll(selector).forEach((element) => {
            if (element.dataset.reveal) return;
            element.dataset.reveal = '1';
            observer.observe(element);
        });
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
        if (state.typing) return;

        if (state.typed || !supported || prefersReducedMotion()) {
            root.classList.remove('typing');
            return;
        }

        state.typing = true;
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
        document.body.appendChild(caret);

        const highlight = new Highlight();
        CSS.highlights.set('typed', highlight);

        const finish = () => {
            state.typing = false;
            document.querySelectorAll('article.typing').forEach((article) => article.classList.remove('typing'));
            CSS.highlights.delete('typed');
            setTimeout(() => caret.remove(), 1200);
        };

        let typed = 0;

        const tick = () => {
            if (typed >= total || !root.isConnected) return finish();

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
                caret.style.left = `${rect.right}px`;
                caret.style.top = `${rect.top}px`;
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

    let path = location.pathname;

    function page() {
        smoothScroll();

        if (location.pathname !== path) {
            path = location.pathname;
            resetScroll();
        }

        if (document.querySelector('article.typing')) typewrite();
        if (document.querySelector('.project:not([data-reveal])')) reveal('.project');
    }

    function start() {
        page();
        new MutationObserver(() => requestAnimationFrame(page)).observe(document.body, {
            childList: true,
            subtree: true,
        });
    }

    if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start);
    else start();
})();
