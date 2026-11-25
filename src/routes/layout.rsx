use crate::data::PERSONAL_INFO;
use crate::icons::{DiscordIcon, GithubIcon, MailIcon};
use crate::nav::NavLink;
use yawn_router::Outlet;

component Layout() {
    view {
        <header class="header">
            <h1>{PERSONAL_INFO.name}</h1>
            <nav>
                <div class="links">
                    <NavLink href="/" section="">"About"</NavLink>
                    <NavLink href="/projects" section="/projects">"Projects"</NavLink>
                    <NavLink href="/blog" section="/blog">"Blog"</NavLink>
                </div>
                <div class="contact">
                    <a href={PERSONAL_INFO.github} aria-label="GitHub" target="_blank" rel="noopener noreferrer">
                        <GithubIcon />
                    </a>
                    <a href={PERSONAL_INFO.discord} aria-label="Discord" target="_blank" rel="noopener noreferrer">
                        <DiscordIcon />
                    </a>
                    <a href="mailto:{PERSONAL_INFO.email}" aria-label="Email" rel="external">
                        <MailIcon />
                    </a>
                </div>
            </nav>
        </header>

        <main>
            <Outlet />
        </main>
    }
}
