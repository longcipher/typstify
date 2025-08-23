use eyre::Result;

use crate::{config::SiteConfig, content::Content};

fn format_directory_name(dir: &str) -> String {
    dir.replace("-", " ")
        .replace("_", " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn get_file_priority(filename: &str) -> i32 {
    match filename {
        "index" | "readme" | "introduction" => 0,
        "getting-started" => 1,
        "installation" => 2,
        "quick-start" => 3,
        _ => 100,
    }
}

pub struct MdBookTemplate {
    config: SiteConfig,
    content_list: Vec<Content>,
}

impl MdBookTemplate {
    pub fn new(config: SiteConfig, content_list: Vec<Content>) -> Self {
        Self {
            config,
            content_list,
        }
    }

    pub fn generate_navigation(&self) -> String {
        let mut nav_html = String::new();

        // Group content by directory structure relative to contents/
        let mut sections: std::collections::BTreeMap<String, Vec<&Content>> =
            std::collections::BTreeMap::new();
        let mut section_index_files: std::collections::HashMap<String, &Content> =
            std::collections::HashMap::new();
        let mut root_files: Vec<&Content> = Vec::new();

        for content in &self.content_list {
            // Get the relative path from contents directory
            let relative_path = content
                .file_path
                .strip_prefix("contents/")
                .unwrap_or(&content.file_path);

            let path_parts: Vec<&str> = relative_path
                .components()
                .filter_map(|comp| match comp {
                    std::path::Component::Normal(name) => name.to_str(),
                    _ => None,
                })
                .collect();

            if path_parts.len() == 1 {
                // File directly in contents/
                let filename = content
                    .file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                // Check if this root file should be used as section index
                // Look for matching directory names
                let matching_dir_exists = self.content_list.iter().any(|other_content| {
                    let other_relative = other_content
                        .file_path
                        .strip_prefix("contents/")
                        .unwrap_or(&other_content.file_path);
                    let other_parts: Vec<&str> = other_relative
                        .components()
                        .filter_map(|comp| match comp {
                            std::path::Component::Normal(name) => name.to_str(),
                            _ => None,
                        })
                        .collect();

                    other_parts.len() > 1 && other_parts[0] == filename
                });

                if matching_dir_exists {
                    // This root file is an index for a section
                    let formatted_dir = format_directory_name(filename);
                    section_index_files.insert(formatted_dir, content);
                } else {
                    // Regular root file
                    root_files.push(content);
                }
            } else {
                // File in subdirectory
                let directory = path_parts[0];
                let formatted_dir = format_directory_name(directory);
                sections.entry(formatted_dir).or_default().push(content);
            }
        }

        // Generate HTML for root level files first (no section header)
        if !root_files.is_empty() {
            root_files.sort_by(|a, b| {
                let a_name = a
                    .file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let b_name = b
                    .file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                let a_priority = get_file_priority(a_name);
                let b_priority = get_file_priority(b_name);

                if a_priority != b_priority {
                    a_priority.cmp(&b_priority)
                } else {
                    a_name.cmp(b_name)
                }
            });

            nav_html.push_str(r#"<ul class="nav-list nav-root">"#);
            for content in root_files {
                nav_html.push_str(&format!(
                    r#"<li class="nav-item">
                        <a href="/{}.html" class="nav-link">
                            {}
                        </a>
                    </li>"#,
                    content.slug(),
                    content.metadata.get_title()
                ));
            }
            nav_html.push_str("</ul>");
        }

        // Generate HTML for sections with their contents
        let mut sorted_sections: Vec<_> = sections.into_iter().collect();
        sorted_sections.sort_by(|a, b| a.0.cmp(&b.0));

        for (section_name, mut contents) in sorted_sections {
            if contents.is_empty() && !section_index_files.contains_key(&section_name) {
                continue;
            }

            // Sort contents within each section by priority and name
            contents.sort_by(|a, b| {
                let a_name = a
                    .file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let b_name = b
                    .file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                let a_priority = get_file_priority(a_name);
                let b_priority = get_file_priority(b_name);

                if a_priority != b_priority {
                    a_priority.cmp(&b_priority)
                } else {
                    a_name.cmp(b_name)
                }
            });

            nav_html.push_str(r#"<div class="nav-section">"#);

            // Section header - make it clickable if there's an index file, with collapse toggle
            if let Some(index_content) = section_index_files.get(&section_name) {
                nav_html.push_str(&format!(
                    r#"<div class="nav-section-header" onclick="toggleSection(this)">
                        <h3 class="nav-section-title">
                            <span class="nav-section-toggle">▶</span>
                            <a href="/{}.html" class="nav-section-link" onclick="event.stopPropagation()">
                                {}
                            </a>
                        </h3>
                    </div>"#,
                    index_content.slug(),
                    section_name
                ));
            } else {
                nav_html.push_str(&format!(
                    r#"<div class="nav-section-header" onclick="toggleSection(this)">
                        <h3 class="nav-section-title">
                            <span class="nav-section-toggle">▶</span>
                            <span>{section_name}</span>
                        </h3>
                    </div>"#
                ));
            }

            // Section contents - wrapped in collapsible container
            if !contents.is_empty() {
                nav_html.push_str(r#"<div class="nav-section-content">"#);
                nav_html.push_str(r#"<ul class="nav-list">"#);
                for content in contents {
                    nav_html.push_str(&format!(
                        r#"<li class="nav-item">
                            <a href="/{}.html" class="nav-link">
                                {}
                            </a>
                        </li>"#,
                        content.slug(),
                        content.metadata.get_title()
                    ));
                }
                nav_html.push_str("</ul>");
                nav_html.push_str("</div>");
            }

            nav_html.push_str("</div>");
        }

        nav_html
    }

    pub fn generate_page(&self, content: &Content, current_slug: &str) -> Result<String> {
        let rendered_content = content.render()?;
        let navigation = self.generate_navigation();

        // Update navigation to mark current page as active
        let navigation = navigation.replace(
            &format!(r#"href="/{current_slug}.html" class="nav-link""#),
            &format!(r#"href="/{current_slug}.html" class="nav-link active""#),
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
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism-tomorrow.min.css">
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
                <button class="menu-toggle" id="menu-toggle">☰</button>
                <div class="breadcrumb">
                    {}
                </div>
            </header>

            <!-- Content Area -->
            <main class="content-area">
                <article class="prose prose-lg max-w-none">
                    {}
                </article>

                <!-- Navigation Buttons -->
                <div class="nav-buttons">
                    {}
                </div>

                <footer class="content-footer">
                    Built with ❤️ using <a href="https://github.com/longcipher/typstify" class="text-primary">Typstify</a>
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
            
            if (window.innerWidth <= 1024 && 
                !sidebar.contains(event.target) && 
                !menuToggle.contains(event.target)) {{
                sidebar.classList.remove('sidebar-open');
            }}
        }});

        // Section collapse/expand functionality
        function toggleSection(headerElement) {{
            const section = headerElement.closest('.nav-section');
            const content = section.querySelector('.nav-section-content');
            const toggle = section.querySelector('.nav-section-toggle');
            
            if (content) {{
                const isCollapsed = content.classList.contains('collapsed');
                
                if (isCollapsed) {{
                    content.classList.remove('collapsed');
                    toggle.classList.add('expanded');
                    toggle.textContent = '▼';
                }} else {{
                    content.classList.add('collapsed');
                    toggle.classList.remove('expanded');
                    toggle.textContent = '▶';
                }}
                
                // Save collapse state to localStorage
                const sectionTitle = section.querySelector('.nav-section-title').textContent.trim();
                localStorage.setItem('nav-section-' + sectionTitle, isCollapsed ? 'expanded' : 'collapsed');
            }}
        }}

        // Restore collapse state from localStorage
        document.addEventListener('DOMContentLoaded', function() {{
            const sections = document.querySelectorAll('.nav-section');
            sections.forEach(section => {{
                const content = section.querySelector('.nav-section-content');
                const toggle = section.querySelector('.nav-section-toggle');
                const titleElement = section.querySelector('.nav-section-title');
                
                if (content && toggle && titleElement) {{
                    const sectionTitle = titleElement.textContent.trim();
                    const savedState = localStorage.getItem('nav-section-' + sectionTitle);
                    
                    if (savedState === 'collapsed') {{
                        content.classList.add('collapsed');
                        toggle.classList.remove('expanded');
                        toggle.textContent = '▶';
                    }} else {{
                        // Default to expanded
                        content.classList.remove('collapsed');
                        toggle.classList.add('expanded');
                        toggle.textContent = '▼';
                    }}
                }}
            }});
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
                    // Also expand parent section if searching
                    if (query) {{
                        const section = listItem.closest('.nav-section');
                        if (section) {{
                            const content = section.querySelector('.nav-section-content');
                            const toggle = section.querySelector('.nav-section-toggle');
                            if (content && content.classList.contains('collapsed')) {{
                                content.classList.remove('collapsed');
                                toggle.classList.add('expanded');
                                toggle.textContent = '▼';
                            }}
                        }}
                    }}
                }} else {{
                    listItem.style.display = query ? 'none' : 'block';
                }}
            }});
        }});

    </script>
</body>
</html>"#,
            content.metadata.get_title(),
            self.config.title,
            self.config.title,
            navigation,
            breadcrumb,
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
            let description = content
                .metadata
                .get_summary()
                .unwrap_or("No description available")
                .chars()
                .take(150)
                .collect::<String>();
            let description = if description.len() == 150 {
                format!("{description}...")
            } else {
                description
            };

            let tags_html = if let Some(tags) = content.metadata.get_tags() {
                let tag_list = tags
                    .iter()
                    .take(3) // Limit to 3 tags
                    .map(|tag| format!(r#"<span class="doc-card-tag">{}</span>"#, tag.as_str()))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!(r#"<div class="doc-card-tags">{tag_list}</div>"#)
            } else {
                String::new()
            };

            content_list_html.push_str(&format!(
                r#"<div class="doc-card">
                    <div class="doc-card-badge">{}</div>
                    <h3 class="doc-card-title">
                        <a href="/{}.html" class="link link-hover text-base-content hover:text-primary transition-colors duration-200">{}</a>
                    </h3>
                    <p class="doc-card-description">{}</p>
                    {}
                </div>"#,
                match content.content_type {
                    crate::content::ContentType::Markdown => "📄 Markdown",
                    crate::content::ContentType::Typst => "📐 Typst",
                },
                content.slug(),
                content.metadata.get_title(),
                description,
                tags_html
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
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism-tomorrow.min.css">
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
                <button class="menu-toggle" id="menu-toggle">☰</button>
                <div class="breadcrumb">
                    <span>Home</span>
                </div>
            </header>

            <!-- Content Area -->
            <main class="content-area">
                <div class="welcome-section">
                    <h1 class="text-4xl font-bold text-primary mb-4">{}</h1>
                    <p class="text-xl text-base-content/80 max-w-2xl">{}</p>
                </div>

                <div class="content-list">
                    <h2 class="text-2xl font-bold text-base-content mb-8">📚 Documentation</h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        {}
                    </div>
                </div>

                <footer class="content-footer">
                    Built with ❤️ using <a href="https://github.com/longcipher/typstify" class="text-primary">Typstify</a>
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
            
            if (window.innerWidth <= 1024 && 
                !sidebar.contains(event.target) && 
                !menuToggle.contains(event.target)) {{
                sidebar.classList.remove('sidebar-open');
            }}
        }});

        // Section collapse/expand functionality
        function toggleSection(headerElement) {{
            const section = headerElement.closest('.nav-section');
            const content = section.querySelector('.nav-section-content');
            const toggle = section.querySelector('.nav-section-toggle');
            
            if (content) {{
                const isCollapsed = content.classList.contains('collapsed');
                
                if (isCollapsed) {{
                    content.classList.remove('collapsed');
                    toggle.classList.add('expanded');
                    toggle.textContent = '▼';
                }} else {{
                    content.classList.add('collapsed');
                    toggle.classList.remove('expanded');
                    toggle.textContent = '▶';
                }}
                
                // Save collapse state to localStorage
                const sectionTitle = section.querySelector('.nav-section-title').textContent.trim();
                localStorage.setItem('nav-section-' + sectionTitle, isCollapsed ? 'expanded' : 'collapsed');
            }}
        }}

        // Restore collapse state from localStorage
        document.addEventListener('DOMContentLoaded', function() {{
            const sections = document.querySelectorAll('.nav-section');
            sections.forEach(section => {{
                const content = section.querySelector('.nav-section-content');
                const toggle = section.querySelector('.nav-section-toggle');
                const titleElement = section.querySelector('.nav-section-title');
                
                if (content && toggle && titleElement) {{
                    const sectionTitle = titleElement.textContent.trim();
                    const savedState = localStorage.getItem('nav-section-' + sectionTitle);
                    
                    if (savedState === 'collapsed') {{
                        content.classList.add('collapsed');
                        toggle.classList.remove('expanded');
                        toggle.textContent = '▶';
                    }} else {{
                        // Default to expanded
                        content.classList.remove('collapsed');
                        toggle.classList.add('expanded');
                        toggle.textContent = '▼';
                    }}
                }}
            }});
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
                    // Also expand parent section if searching
                    if (query) {{
                        const section = listItem.closest('.nav-section');
                        if (section) {{
                            const content = section.querySelector('.nav-section-content');
                            const toggle = section.querySelector('.nav-section-toggle');
                            if (content && content.classList.contains('collapsed')) {{
                                content.classList.remove('collapsed');
                                toggle.classList.add('expanded');
                                toggle.textContent = '▼';
                            }}
                        }}
                    }}
                }} else {{
                    listItem.style.display = query ? 'none' : 'block';
                }}
            }});
        }});
    </script>
</body>
</html>"#,
            self.config.title,
            self.config.title,
            navigation,
            self.config.title,
            self.config.description,
            content_list_html
        );

        Ok(html)
    }

    fn generate_breadcrumb(&self, _content: &Content) -> String {
        // Always just show "Home" link for breadcrumb
        String::from(r#"<a href="/index.html">Home</a>"#)
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
                    r#"<a href="/{}.html" class="nav-button">
                        <span class="nav-button-icon">←</span>
                        <div class="nav-button-content">
                            <span class="nav-button-label">Previous</span>
                            <span class="nav-button-title">{}</span>
                        </div>
                    </a>"#,
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
                    r#"<a href="/{}.html" class="nav-button nav-button-next">
                        <div class="nav-button-content">
                            <span class="nav-button-label">Next</span>
                            <span class="nav-button-title">{}</span>
                        </div>
                        <span class="nav-button-icon">→</span>
                    </a>"#,
                    next_content.slug(),
                    next_content.metadata.get_title()
                ));
            }
        }

        buttons
    }
}
