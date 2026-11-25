use crate::data::POSTS;
use yawn_router::Link;

component Index() {
    yawn::ui::set_title("Blog | meetzli");
    let rows: Vec<(&'static str, &'static str, &'static str, bool)> = POSTS
        .iter()
        .filter(|post| !post.draft || cfg!(debug_assertions))
        .map(|post| (post.slug, post.title, post.date, post.draft))
        .collect();
    let none = rows.is_empty();
    let rows = signal(rows);

    view {
        <article>
            if none {
                <p>"Nothing here yet."</p>
            } else {
                <ul class="post-list">
                    for row in rows.get() key row.0 {
                        <li>
                            <span class="post-date">{row.2}</span>
                            <Link href={format!("/blog/{}", row.0)}>{row.1}</Link>
                            if row.3 {
                                <span class="post-draft">"draft"</span>
                            }
                        </li>
                    }
                </ul>
            }
        </article>
    }
}
