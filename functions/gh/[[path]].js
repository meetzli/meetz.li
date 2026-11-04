import init, { handle, image_version } from '../../tools/github-share/pkg/github_share.js';
import wasm from '../../tools/github-share/pkg/github_share_bg.wasm';

let ready;

export async function onRequest({ request, env, waitUntil }) {
    ready ??= init({ module_or_path: wasm });
    await ready;
    const url = new URL(request.url);
    const protocol = request.headers.get('x-forwarded-proto');
    if (protocol === 'https' || protocol === 'http') url.protocol = `${protocol}:`;
    const cacheable = request.method === 'GET' || request.method === 'HEAD';
    const cacheUrl = new URL(url);
    cacheUrl.searchParams.set('_renderer', image_version());
    const key = new Request(cacheUrl, { method: 'GET' });
    const cache = caches.default;
    let response = cacheable ? await cache.match(key) : null;
    if (!response) {
        response = await handle(url.href, request.method, env.GITHUB_TOKEN || '');
        if (cacheable && response.ok) waitUntil(cache.put(key, response.clone()));
    }
    return request.method === 'HEAD' ? new Response(null, response) : response;
}
