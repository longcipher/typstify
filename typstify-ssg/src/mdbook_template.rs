use crate::config::LegacySiteConfig;
use crate::content::Content;
use eyre::Result;

pub struct MdBookTemplate {
    config: LegacySiteConfig,
    content_list: Vec<Content>,
}

impl MdBookTemplate {
    pub fn new(config: LegacySiteConfig, content_list: Vec<Content>) -> Self {
        Self {
            config,
            content_list,
        }
    }

    pub fn generate_navigation(&self) -> String {
        let mut nav_html = String::new();

        // Separate root-level content from grouped content
        let mut root_content: Vec<&Content> = Vec::new();
        let mut sections: std::collections::BTreeMap<String, Vec<&Content>> =
            std::collections::BTreeMap::new();

        for content in &self.content_list {
            // Get the relative path from contents directory
            let relative_path = content
                .file_path
                .strip_prefix("contents/")
                .unwrap_or(&content.file_path);

            if let Some(parent) = relative_path.parent() {
                if parent.as_os_str().is_empty() {
                    // Files directly in contents/ go to root level
                    root_content.push(content);
                } else {
                    // Files in subdirectories use the directory name
                    let section = parent
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Other")
                        .replace("-", " ")
                        .replace("_", " ")
                        .split_whitespace()
                        .map(|word| {
                            let mut chars = word.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(first) => {
                                    first.to_uppercase().collect::<String>() + chars.as_str()
                                }
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(" ");
                    
                    sections.entry(section).or_default().push(content);
                }
            } else {
                // Fallback: add to root
                root_content.push(content);
            }
        }

        // Generate root-level navigation first
        if !root_content.is_empty() {
            // Sort root content by filename
            root_content.sort_by(|a, b| {
                let a_path = a.file_path.file_name().unwrap_or_default();
                let b_path = b.file_path.file_name().unwrap_or_default();
                a_path.cmp(b_path)
            });

            nav_html.push_str(r#"<div class="nav-root">
                    <ul class="nav-list">"#);

            for content in root_content {
                nav_html.push_str(&format!(
                    r#"<li class="nav-item">
                            <a href="{}.html" class="nav-link">{}</a>
                        </li>"#,
                    content.slug(),
                    content.metadata.get_title()
                ));
            }

            nav_html.push_str("</ul></div>");
        }

        // Generate grouped sections
        for (section_name, mut contents) in sections {
            // Sort contents within each section by filename
            contents.sort_by(|a, b| {
                let a_path = a.file_path.file_name().unwrap_or_default();
                let b_path = b.file_path.file_name().unwrap_or_default();
                a_path.cmp(b_path)
            });

            nav_html.push_str(&format!(
                r#"<div class="nav-section">
                    <h3 class="nav-section-title">{}</h3>
                    <ul class="nav-list">"#,
                section_name
            ));

            for content in contents {
                let is_active = false; // TODO: determine based on current page
                let active_class = if is_active { " active" } else { "" };

                nav_html.push_str(&format!(
                    r#"<li class="nav-item">
                        <a href="/{}.html" class="nav-link{}">
                            {}
                        </a>
                    </li>"#,
                    content.slug(),
                    active_class,
                    content.metadata.get_title()
                ));
            }

            nav_html.push_str("</ul></div>");
        }

        nav_html
    }

    pub fn generate_page(&self, content: &Content, current_slug: &str) -> Result<String> {
        let rendered_content = content.render()?;
        let navigation = self.generate_navigation();

        // Update navigation to mark current page as active
        let navigation = navigation.replace(
            &format!(r#"href="/{}.html" class="nav-link""#, current_slug),
            &format!(r#"href="/{}.html" class="nav-link active""#, current_slug),
        );

        let breadcrumb = self.generate_breadcrumb(content);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{} - {}</title>
    <link rel="stylesheet" href="/style/output.css">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-core.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/plugins/autoloader/prism-autoloader.min.js"></script>
</head>
<body>
    <div class="book-container">
        <!-- Sidebar Navigation -->
        <nav class="sidebar" id="sidebar">
            <div class="sidebar-header">
                <a href="/index.html" class="sidebar-title">{}</a>
            </div>
            
            <div class="search-box">
                <input type="text" class="search-input" placeholder="Search documentation...">
            </div>
            
            <div class="sidebar-nav">
                {}
            </div>
        </nav>

        <!-- Main Content -->
        <div class="main-content">
            <!-- Top Bar -->
            <header class="topbar">
                <button class="menu-toggle" id="menu-toggle">
                    ☰
                </button>
                <div class="breadcrumb">
                    {}
                </div>
            </header>

            <!-- Content Area -->
            <main class="content-area">
                <div class="content-header">
                    <h1 class="content-title">{}</h1>
                    <p class="content-subtitle">{}</p>
                </div>

                <div class="prose">
                    {}
                </div>

                <!-- Navigation Buttons -->
                <div class="nav-buttons">
                    {}
                </div>

                <footer class="content-footer">
                    Built with ❤️ using <a href="https://github.com/longcipher/typstify">Typstify</a>
                </footer>
            </main>
        </div>
    </div>

    <script>
        // Mobile menu toggle
        document.getElementById('menu-toggle').addEventListener('click', function() {{
            const sidebar = document.getElementById('sidebar');
            sidebar.classList.toggle('sidebar-open');
        }});

        // Close sidebar when clicking outside on mobile
        document.addEventListener('click', function(event) {{
            const sidebar = document.getElementById('sidebar');
            const menuToggle = document.getElementById('menu-toggle');
            
            if (window.innerWidth <= 768 && 
                !sidebar.contains(event.target) && 
                !menuToggle.contains(event.target)) {{
                sidebar.classList.remove('sidebar-open');
            }}
        }});

        // Search functionality placeholder
        const searchInput = document.querySelector('.search-input');
        searchInput.addEventListener('input', function(event) {{
            const query = event.target.value.toLowerCase();
            const navLinks = document.querySelectorAll('.nav-link');
            
            navLinks.forEach(link => {{
                const text = link.textContent.toLowerCase();
                const listItem = link.closest('.nav-item');
                if (text.includes(query)) {{
                    listItem.style.display = 'block';
                }} else {{
                    listItem.style.display = query ? 'none' : 'block';
                }}
            }});
        }});
    </script>
</body>
</html>"#,
            content.metadata.get_title(),
            self.config.website_title,
            self.config.website_title,
            navigation,
            breadcrumb,
            content.metadata.get_title(),
            content.metadata.get_summary().unwrap_or_default(),
            rendered_content,
            self.generate_nav_buttons(content)
        );

        Ok(html)
    }

    pub fn generate_index_page(&self) -> Result<String> {
        let navigation = self.generate_navigation();

        // Generate content list for index
        let mut content_list_html = String::new();
        for content in &self.content_list {
            content_list_html.push_str(&format!(
                r#"<div class="content-card">
                    <h3><a href="/{}.html">{}</a></h3>
                    <p>{}</p>
                    <div class="content-meta">
                        <span class="content-type">{}</span>
                        {}
                    </div>
                </div>"#,
                content.slug(),
                content.metadata.get_title(),
                content
                    .metadata
                    .get_summary()
                    .unwrap_or("No description available"),
                match content.content_type {
                    crate::content::ContentType::Markdown => "📄 Markdown",
                    crate::content::ContentType::Typst => "📐 Typst",
                },
                if let Some(tags) = content.metadata.get_tags() {
                    format!(
                        "<span class=\"tags\">{}</span>",
                        tags.iter()
                            .map(|t| t.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                } else {
                    String::new()
                }
            ));
        }

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{}</title>
    <link rel="stylesheet" href="/style/output.css">
    <style>
        .content-card {{
            background-color: var(--bg-secondary);
            border: 1px solid var(--dracula-current-line);
            border-radius: 0.5rem;
            padding: 1.5rem;
            margin-bottom: 1.5rem;
            transition: transform 0.2s ease, box-shadow 0.2s ease;
        }}
        
        .content-card:hover {{
            transform: translateY(-2px);
            box-shadow: var(--shadow-lg);
        }}
        
        .content-card h3 {{
            margin: 0 0 0.5rem 0;
            font-size: 1.25rem;
        }}
        
        .content-card h3 a {{
            color: var(--accent-primary);
            text-decoration: none;
        }}
        
        .content-card h3 a:hover {{
            color: var(--accent-secondary);
        }}
        
        .content-card p {{
            margin: 0 0 1rem 0;
            color: var(--text-secondary);
        }}
        
        .content-meta {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            font-size: 0.875rem;
            color: var(--text-secondary);
        }}
        
        .content-type {{
            font-weight: 600;
        }}
        
        .tags {{
            color: var(--accent-secondary);
        }}
        
        .welcome-section {{
            background: linear-gradient(135deg, var(--accent-primary), var(--accent-secondary));
            color: var(--bg-primary);
            padding: 3rem 2rem;
            border-radius: 1rem;
            margin-bottom: 3rem;
            text-align: center;
        }}
        
        .welcome-section h1 {{
            font-size: 3rem;
            margin: 0 0 1rem 0;
            color: var(--bg-primary);
        }}
        
        .welcome-section p {{
            font-size: 1.2rem;
            margin: 0;
            color: var(--bg-primary);
            opacity: 0.9;
        }}
    </style>
</head>
<body>
    <div class="book-container">
        <!-- Sidebar Navigation -->
        <nav class="sidebar" id="sidebar">
            <div class="sidebar-header">
                <a href="/index.html" class="sidebar-title">{}</a>
            </div>
            
            <div class="search-box">
                <input type="text" class="search-input" placeholder="Search documentation...">
            </div>
            
            <div class="sidebar-nav">
                {}
            </div>
        </nav>

        <!-- Main Content -->
        <div class="main-content">
            <!-- Top Bar -->
            <header class="topbar">
                <button class="menu-toggle" id="menu-toggle">
                    ☰
                </button>
                <div class="breadcrumb">
                    <a href="/index.html">Home</a>
                </div>
            </header>

            <!-- Content Area -->
            <main class="content-area">
                <div class="welcome-section">
                    <h1>{}</h1>
                    <p>{}</p>
                </div>

                <div class="content-list">
                    <h2>Documentation</h2>
                    {}
                </div>

                <footer class="content-footer">
                    Built with ❤️ using <a href="https://github.com/longcipher/typstify">Typstify</a>
                </footer>
            </main>
        </div>
    </div>

    <script>
        // Mobile menu toggle
        document.getElementById('menu-toggle').addEventListener('click', function() {{
            const sidebar = document.getElementById('sidebar');
            sidebar.classList.toggle('sidebar-open');
        }});

        // Close sidebar when clicking outside on mobile
        document.addEventListener('click', function(event) {{
            const sidebar = document.getElementById('sidebar');
            const menuToggle = document.getElementById('menu-toggle');
            
            if (window.innerWidth <= 768 && 
                !sidebar.contains(event.target) && 
                !menuToggle.contains(event.target)) {{
                sidebar.classList.remove('sidebar-open');
            }}
        }});

        // Search functionality
        const searchInput = document.querySelector('.search-input');
        searchInput.addEventListener('input', function(event) {{
            const query = event.target.value.toLowerCase();
            const navLinks = document.querySelectorAll('.nav-link');
            
            navLinks.forEach(link => {{
                const text = link.textContent.toLowerCase();
                const listItem = link.closest('.nav-item');
                if (text.includes(query)) {{
                    listItem.style.display = 'block';
                }} else {{
                    listItem.style.display = query ? 'none' : 'block';
                }}
            }});
        }});
    </script>
</body>
</html>"#,
            self.config.website_title,
            self.config.website_title,
            navigation,
            self.config.website_title,
            self.config.website_tagline,
            content_list_html
        );

        Ok(html)
    }

    fn generate_breadcrumb(&self, content: &Content) -> String {
        let path_parts: Vec<&str> = content
            .file_path
            .parent()
            .unwrap_or(&content.file_path)
            .components()
            .filter_map(|comp| match comp {
                std::path::Component::Normal(name) => name.to_str(),
                _ => None,
            })
            .collect();

        let mut breadcrumb = String::from(r#"<a href="/index.html">Home</a>"#);

        if path_parts.len() > 1 && path_parts[0] == "contents" {
            for part in &path_parts[1..] {
                let formatted_part = part.replace("-", " ").replace("_", " ");
                breadcrumb.push_str(&format!(r#" / <span>{}</span>"#, formatted_part));
            }
        }

        breadcrumb
    }

    fn generate_nav_buttons(&self, current_content: &Content) -> String {
        // Find current content index
        let current_index = self
            .content_list
            .iter()
            .position(|c| c.slug() == current_content.slug());

        let mut buttons = String::new();

        if let Some(index) = current_index {
            // Previous button
            if index > 0 {
                let prev_content = &self.content_list[index - 1];
                buttons.push_str(&format!(
                    r#"<a href="/{}.html" class="nav-button">← {}</a>"#,
                    prev_content.slug(),
                    prev_content.metadata.get_title()
                ));
            } else {
                buttons.push_str(r#"<span></span>"#);
            }

            // Next button
            if index < self.content_list.len() - 1 {
                let next_content = &self.content_list[index + 1];
                buttons.push_str(&format!(
                    r#"<a href="/{}.html" class="nav-button">{} →</a>"#,
                    next_content.slug(),
                    next_content.metadata.get_title()
                ));
            }
        }

        buttons
    }
}
