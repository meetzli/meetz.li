use crate::data::POSTS;
use crate::markdown;
use yawn_router::{use_params, Link};

component Slug() {
    let params = use_params();
    let found = memo(move || {
        let slug = params.get("slug");
        POSTS.iter().position(|post| post.slug == slug && (!post.draft || cfg!(debug_assertions)))
    });
    effect(move || {
        yawn::ui::set_title(&match found.get() {
            Some(i) => format!("{} | meetzli", POSTS[i].title),
            None => "Not found | meetzli".to_string(),
        })
    });

    view {
        if let Some(i) = found.get() {
            <article class="post">
                <h1>{POSTS[i].title}</h1>
                <p class="post-meta">{POSTS[i].date}</p>
                <div class="post-body" prop:innerHTML={markdown::render(POSTS[i].body)}></div>
            </article>
        } else {
            <article>
                <p>"No such post. Back to the " <Link href="/blog">"blog"</Link> "."</p>
            </article>
        }
    }
}
