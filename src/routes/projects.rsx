use crate::data::PROJECTS;
use crate::typography::{bind_end, tie};

component Projects() {
    yawn::ui::set_title("Projects | meetzli");
    let projects: Vec<&'static crate::data::Project> = PROJECTS.iter().rev().collect();

    view {
        <article>
            for project in projects.clone() key project.name {
                <section class="project">
                    <span class="when">{project.date}</span>
                    <h2>
                        <a href={project.link} target="_blank" rel="noopener noreferrer">{project.name}</a>
                    </h2>
                    <p class="meta">{project.tech}</p>
                    <p>{bind_end(&tie(project.description))}</p>
                    <a class="project-preview" href={project.link} target="_blank" rel="noopener noreferrer">
                        <img src={project.image} alt="Preview of {project.name}" loading="lazy" width="1200" height="600" />
                    </a>
                </section>
            }
        </article>
    }
}
