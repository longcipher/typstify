use eyre::Result;

use crate::{config::LegacySiteConfig, content::Content};

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

            nav_html.push_str(
                r#"<div class="nav-root">
                    <ul class="nav-list">"#,
            );

            for content in root_content {
                nav_html.push_str(&format!(
                    r#"<li class="nav-item">
                            <a href="/{}" class="nav-link">{}</a>
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
                        <a href="/{}" class="nav-link{}">
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
            &format!(r#"href="/{}" class="nav-link""#, current_slug),
            &format!(r#"href="/{}" class="nav-link active""#, current_slug),
        );

        let breadcrumb = self.generate_breadcrumb(content);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{} - {}</title>
    <link rel="stylesheet" href="/assets/search.css">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-core.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/plugins/autoloader/prism-autoloader.min.js"></script>
    <style>
        :root {{
            --bg-primary: #ffffff;
            --bg-secondary: #f7f7f5;
            --text-primary: #37352f;
            --text-secondary: #787774;
            --accent-primary: #2eaadc;
            --border-color: #e9e9e7;
            --sidebar-width: 260px;
            --content-width: 740px;
            --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, "Apple Color Emoji", Arial, sans-serif;
        }}

        [data-theme="dark"] {{
            --bg-primary: #191919;
            --bg-secondary: #202020;
            --text-primary: #d4d4d4;
            --text-secondary: #9b9a97;
            --accent-primary: #2eaadc;
            --border-color: #2f2f2f;
        }}

        body {{
            background-color: var(--bg-primary);
            color: var(--text-primary);
            font-family: var(--font-sans);
            margin: 0;
            line-height: 1.6;
            -webkit-font-smoothing: antialiased;
        }}

        .book-container {{
            display: flex;
            min-height: 100vh;
        }}

        .sidebar {{
            width: var(--sidebar-width);
            background-color: var(--bg-secondary);
            border-right: 1px solid var(--border-color);
            position: fixed;
            height: 100vh;
            overflow-y: auto;
            transition: transform 0.3s ease;
            z-index: 50;
        }}

        .sidebar-header {{
            padding: 1.5rem;
            margin-bottom: 1rem;
        }}

        .sidebar-title {{
            font-size: 1rem;
            font-weight: 600;
            color: var(--text-primary);
            text-decoration: none;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }}

        .sidebar-nav {{
            padding: 0 0.75rem 1.5rem;
        }}

        .nav-section {{
            margin-bottom: 1.5rem;
        }}

        .nav-section-title {{
            font-size: 0.75rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-secondary);
            margin: 0 0.75rem 0.5rem;
            font-weight: 600;
        }}

        .nav-list {{
            list-style: none;
            padding: 0;
            margin: 0;
        }}

        .nav-item {{
            margin-bottom: 1px;
        }}

        .nav-link {{
            display: block;
            padding: 0.35rem 0.75rem;
            color: var(--text-secondary);
            text-decoration: none;
            border-radius: 4px;
            transition: all 0.1s;
            font-size: 0.9rem;
        }}

        .nav-link:hover {{
            background-color: rgba(0, 0, 0, 0.04);
            color: var(--text-primary);
        }}

        .nav-link.active {{
            background-color: rgba(0, 0, 0, 0.04);
            color: var(--text-primary);
            font-weight: 500;
        }}

        .main-content {{
            flex: 1;
            margin-left: var(--sidebar-width);
            min-width: 0;
            background-color: var(--bg-primary);
        }}

        .topbar {{
            position: sticky;
            top: 0;
            z-index: 40;
            background-color: rgba(255, 255, 255, 0.8);
            backdrop-filter: blur(10px);
            border-bottom: 1px solid var(--border-color);
            padding: 0.75rem 2rem;
            display: flex;
            align-items: center;
            gap: 1rem;
        }}

        [data-theme="dark"] .topbar {{
            background-color: rgba(25, 25, 25, 0.8);
        }}
        
        [data-theme="dark"] .nav-link:hover, 
        [data-theme="dark"] .nav-link.active {{
            background-color: rgba(255, 255, 255, 0.06);
        }}

        body {{
            background-color: var(--bg-primary);
            color: var(--text-primary);
            font-family: var(--font-sans);
            margin: 0;
            line-height: 1.6;
            -webkit-font-smoothing: antialiased;
        }}

        .right-actions {{
            margin-left: auto;
            display: flex;
            align-items: center;
            gap: 1rem;
            flex-shrink: 1;
            min-width: 0;
        }}

        .search-box {{
            position: relative;
            flex: 1 1 auto;
            min-width: 0;
        }}

        .search-input {{
            padding: 0.4rem 0.8rem;
            border-radius: 4px;
            border: 1px solid var(--border-color);
            background: var(--bg-secondary);
            color: var(--text-primary);
            width: 200px;
            min-width: 100px;
            max-width: 100%;
            font-size: 0.9rem;
            transition: all 0.2s;
            box-sizing: border-box;
        }}

        .search-input:focus {{
            width: 280px;
            max-width: 100%;
            outline: none;
            border-color: var(--accent-primary);
            background: var(--bg-primary);
        }}

        .theme-toggle {{
            background: none;
            border: none;
            color: var(--text-secondary);
            cursor: pointer;
            padding: 0.4rem;
            border-radius: 4px;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: all 0.2s;
            flex-shrink: 0;
        }}

        .theme-toggle:hover {{
            background-color: var(--bg-secondary);
            color: var(--text-primary);
        }}

        .content-area {{
            padding: 3rem 2rem 6rem;
            max-width: var(--content-width);
            margin: 0 auto;
        }}

        .content-header {{
            margin-bottom: 3rem;
        }}

        .content-title {{
            font-size: 2.5rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
            line-height: 1.2;
            letter-spacing: -0.02em;
        }}

        .content-subtitle {{
            font-size: 1.25rem;
            color: var(--text-secondary);
            font-weight: 400;
        }}

        .prose {{
            color: var(--text-primary);
            font-size: 1.05rem;
        }}

        .prose h1, .prose h2, .prose h3, .prose h4 {{
            color: var(--text-primary);
            margin-top: 2em;
            margin-bottom: 0.75em;
            line-height: 1.3;
            font-weight: 600;
            letter-spacing: -0.01em;
        }}
        
        .prose h2 {{
            font-size: 1.75rem;
            border-bottom: 1px solid var(--border-color);
            padding-bottom: 0.3em;
        }}

        .prose p {{
            margin-bottom: 1.5em;
            line-height: 1.7;
        }}

        .prose a {{
            color: var(--text-primary);
            text-decoration: none;
            border-bottom: 1px solid var(--text-secondary);
            transition: border-color 0.2s;
        }}

        .prose a:hover {{
            border-bottom-color: var(--accent-primary);
            color: var(--accent-primary);
        }}

        .prose code {{
            background-color: rgba(135, 131, 120, 0.15);
            color: #EB5757;
            padding: 0.2em 0.4em;
            border-radius: 3px;
            font-size: 0.85em;
            font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, Courier, monospace;
        }}
        
        @media (prefers-color-scheme: dark) {{
            .prose code {{
                color: #ff6b6b;
            }}
        }}

        .prose pre {{
            background-color: var(--bg-secondary);
            padding: 1.25rem;
            border-radius: 4px;
            overflow-x: auto;
            margin-bottom: 1.5em;
            border: 1px solid var(--border-color);
        }}

        .prose pre code {{
            background-color: transparent;
            padding: 0;
            color: inherit;
            font-size: 0.9em;
        }}

        .prose blockquote {{
            border-left: 3px solid var(--text-primary);
            padding-left: 1.25rem;
            margin-left: 0;
            color: var(--text-primary);
            font-style: italic;
            font-size: 1.1em;
        }}

        .prose ul, .prose ol {{
            padding-left: 1.5em;
            margin-bottom: 1.5em;
        }}

        .prose li {{
            margin-bottom: 0.5em;
        }}

        .nav-buttons {{
            display: flex;
            justify-content: space-between;
            margin-top: 4rem;
            padding-top: 2rem;
            border-top: 1px solid var(--border-color);
        }}

        .nav-button {{
            display: inline-flex;
            align-items: center;
            padding: 0.5rem 1rem;
            color: var(--text-secondary);
            text-decoration: none;
            border-radius: 4px;
            transition: all 0.2s;
            font-size: 0.9rem;
        }}

        .nav-button:hover {{
            background-color: var(--bg-secondary);
            color: var(--text-primary);
        }}

        .content-footer {{
            margin-top: 4rem;
            padding-top: 2rem;
            border-top: 1px solid var(--border-color);
            text-align: center;
            color: var(--text-secondary);
            font-size: 0.875rem;
        }}

        .breadcrumb a:hover {{
            color: var(--text-primary);
        }}

        .breadcrumb-home {{
            display: inline-flex;
            align-items: center;
            justify-content: center;
            padding: 4px;
            border-radius: 4px;
            color: var(--text-secondary);
            transition: all 0.2s;
        }}
        
        .breadcrumb-home:hover {{
            background-color: var(--bg-secondary);
            color: var(--text-primary);
        }}

        @media (max-width: 768px) {{
            .sidebar {{
                transform: translateX(-100%);
                box-shadow: 0 0 20px rgba(0,0,0,0.1);
            }}

            .sidebar.sidebar-open {{
                transform: translateX(0);
            }}

            .main-content {{
                margin-left: 0;
            }}

            .menu-toggle {{
                display: block;
            }}

            .content-area {{
                padding: 2rem 1.5rem;
            }}

            .search-box {{
                position: static !important;
            }}

            .search-input:focus {{
                width: calc(100% - 6.5rem);
                max-width: none !important;
                position: absolute;
                left: 1rem;
                top: 50%;
                transform: translateY(-50%);
                z-index: 100;
                box-sizing: border-box;
            }}
        }}

        /* Hero Section */
        .hero-section {{
            text-align: center;
            padding: 6rem 1rem 4rem;
            max-width: 800px;
            margin: 0 auto;
        }}
        
        .hero-title {{
            font-size: 3.5rem;
            font-weight: 800;
            margin-bottom: 1rem;
            letter-spacing: -0.03em;
            background: linear-gradient(135deg, var(--text-primary) 0%, var(--text-secondary) 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }}
        
        .hero-subtitle {{
            font-size: 1.25rem;
            color: var(--text-secondary);
            line-height: 1.5;
            max-width: 600px;
            margin: 0 auto 2rem;
        }}

        .content-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
            gap: 1.5rem;
            margin-top: 2rem;
        }}

        .content-card {{
            background-color: var(--bg-primary);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 1.5rem;
            transition: all 0.2s ease;
            display: flex;
            flex-direction: column;
            height: 100%;
        }}
        
        .content-card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 10px 30px -10px rgba(0, 0, 0, 0.1);
            border-color: var(--text-secondary);
        }}
        
        .content-card h3 {{
            margin: 0 0 0.5rem 0;
            font-size: 1.1rem;
            font-weight: 600;
        }}
        
        .content-card h3 a {{
            color: var(--text-primary);
            text-decoration: none;
        }}
        
        .content-card h3 a::after {{
            content: '';
            position: absolute;
            inset: 0;
        }}
        
        .content-card p {{
            margin: 0 0 1rem 0;
            color: var(--text-secondary);
            font-size: 0.9rem;
            flex-grow: 1;
            line-height: 1.5;
        }}
        
        .content-meta {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            font-size: 0.75rem;
            color: var(--text-secondary);
            margin-top: auto;
            padding-top: 1rem;
            border-top: 1px solid var(--border-color);
        }}
        
        .content-type {{
            font-weight: 500;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}
        
        .tags {{
            background-color: var(--bg-secondary);
            padding: 0.2em 0.5em;
            border-radius: 4px;
        }}
    </style>
    <script>
        // Theme initialization
        (function() {{
            const savedTheme = localStorage.getItem('theme');
            const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
            if (savedTheme === 'dark' || (!savedTheme && systemDark)) {{
                document.documentElement.setAttribute('data-theme', 'dark');
            }} else {{
                document.documentElement.setAttribute('data-theme', 'light');
            }}
        }})();
    </script>
</head>
<body>
    <div class="book-container">
        <!-- Sidebar Navigation -->
        <nav class="sidebar" id="sidebar">
            <div class="sidebar-header">
                <a href="/" class="sidebar-title">{}</a>
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
                <div class="right-actions">
                    <div class="search-box" id="search-container">
                        <input type="text" id="search-input" class="search-input" placeholder="Search...">
                        <div id="search-results" class="search-results"></div>
                    </div>
                    <button class="theme-toggle" id="theme-toggle" aria-label="Toggle theme">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <circle cx="12" cy="12" r="5"></circle>
                            <line x1="12" y1="1" x2="12" y2="3"></line>
                            <line x1="12" y1="21" x2="12" y2="23"></line>
                            <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                            <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                            <line x1="1" y1="12" x2="3" y2="12"></line>
                            <line x1="21" y1="12" x2="23" y2="12"></line>
                            <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                            <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
                        </svg>
                    </button>
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

        // Theme toggle
        document.getElementById('theme-toggle').addEventListener('click', function() {{
            const current = document.documentElement.getAttribute('data-theme');
            const next = current === 'dark' ? 'light' : 'dark';
            document.documentElement.setAttribute('data-theme', next);
            localStorage.setItem('theme', next);
        }});
    </script>
    <script src="/assets/search.js"></script>
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
                    <h3><a href="/{}">{}</a></h3>
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
    <link rel="stylesheet" href="/assets/search.css">
    <style>
        :root {{
            --bg-primary: #ffffff;
            --bg-secondary: #f7f7f5;
            --text-primary: #37352f;
            --text-secondary: #787774;
            --accent-primary: #2eaadc;
            --border-color: #e9e9e7;
            --sidebar-width: 260px;
            --content-width: 740px;
            --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, "Apple Color Emoji", Arial, sans-serif;
        }}

        [data-theme="dark"] {{
            --bg-primary: #191919;
            --bg-secondary: #202020;
            --text-primary: #d4d4d4;
            --text-secondary: #9b9a97;
            --accent-primary: #2eaadc;
            --border-color: #2f2f2f;
        }}

        body {{
            background-color: var(--bg-primary);
            color: var(--text-primary);
            font-family: var(--font-sans);
            margin: 0;
            line-height: 1.6;
            -webkit-font-smoothing: antialiased;
        }}

        /* Sidebar styles (same as generate_page) */
        .book-container {{
            display: flex;
            min-height: 100vh;
        }}

        .sidebar {{
            width: var(--sidebar-width);
            background-color: var(--bg-secondary);
            border-right: 1px solid var(--border-color);
            position: fixed;
            height: 100vh;
            overflow-y: auto;
            transition: transform 0.3s ease;
            z-index: 50;
        }}

        .sidebar-header {{
            padding: 1.5rem;
            margin-bottom: 1rem;
        }}

        .sidebar-title {{
            font-size: 1rem;
            font-weight: 600;
            color: var(--text-primary);
            text-decoration: none;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }}

        .sidebar-nav {{
            padding: 0 0.75rem 1.5rem;
        }}

        .nav-section {{
            margin-bottom: 1.5rem;
        }}

        .nav-section-title {{
            font-size: 0.75rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-secondary);
            margin: 0 0.75rem 0.5rem;
            font-weight: 600;
        }}

        .nav-list {{
            list-style: none;
            padding: 0;
            margin: 0;
        }}

        .nav-item {{
            margin-bottom: 1px;
        }}

        .nav-link {{
            display: block;
            padding: 0.35rem 0.75rem;
            color: var(--text-secondary);
            text-decoration: none;
            border-radius: 4px;
            transition: all 0.1s;
            font-size: 0.9rem;
        }}

        .nav-link:hover {{
            background-color: rgba(0, 0, 0, 0.04);
            color: var(--text-primary);
        }}

        .nav-link.active {{
            background-color: rgba(0, 0, 0, 0.04);
            color: var(--text-primary);
            font-weight: 500;
        }}

        .main-content {{
            flex: 1;
            margin-left: var(--sidebar-width);
            min-width: 0;
            background-color: var(--bg-primary);
        }}

        .topbar {{
            position: sticky;
            top: 0;
            z-index: 40;
            background-color: rgba(255, 255, 255, 0.8);
            backdrop-filter: blur(10px);
            border-bottom: 1px solid var(--border-color);
            padding: 0.75rem 2rem;
            display: flex;
            align-items: center;
            gap: 1rem;
        }}

        [data-theme="dark"] {{
            --bg-primary: #191919;
            --bg-secondary: #202020;
            --text-primary: #d4d4d4;
            --text-secondary: #9b9a97;
            --accent-primary: #2eaadc;
            --border-color: #2f2f2f;
        }}

        [data-theme="dark"] .topbar {{
            background-color: rgba(25, 25, 25, 0.8);
        }}
        
        [data-theme="dark"] .nav-link:hover, 
        [data-theme="dark"] .nav-link.active {{
            background-color: rgba(255, 255, 255, 0.06);
        }}

        .menu-toggle {{
            display: none;
            background: none;
            border: none;
            font-size: 1.25rem;
            color: var(--text-primary);
            cursor: pointer;
            padding: 0;
        }}

        .breadcrumb {{
            color: var(--text-secondary);
            font-size: 0.875rem;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }}

        .breadcrumb a {{
            color: var(--text-secondary);
            text-decoration: none;
            transition: color 0.2s;
        }}

        .breadcrumb a:hover {{
            color: var(--text-primary);
        }}

        .breadcrumb-home {{
            display: inline-flex;
            align-items: center;
            justify-content: center;
            padding: 4px;
            border-radius: 4px;
            color: var(--text-secondary);
            transition: all 0.2s;
        }}
        
        .breadcrumb-home:hover {{
            background-color: var(--bg-secondary);
            color: var(--text-primary);
        }}

        .right-actions {{
            margin-left: auto;
            display: flex;
            align-items: center;
            gap: 1rem;
            flex-shrink: 1;
            min-width: 0;
        }}

        .search-box {{
            position: relative;
            flex: 1 1 auto;
            min-width: 0;
        }}

        .search-input {{
            padding: 0.4rem 0.8rem;
            border-radius: 4px;
            border: 1px solid var(--border-color);
            background: var(--bg-secondary);
            color: var(--text-primary);
            width: 200px;
            min-width: 100px;
            max-width: 100%;
            font-size: 0.9rem;
            transition: all 0.2s;
            box-sizing: border-box;
        }}

        .search-input:focus {{
            width: 280px;
            max-width: 100%;
            outline: none;
            border-color: var(--accent-primary);
            background: var(--bg-primary);
        }}

        .theme-toggle {{
            background: none;
            border: none;
            color: var(--text-secondary);
            cursor: pointer;
            padding: 0.4rem;
            border-radius: 4px;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: all 0.2s;
            flex-shrink: 0;
        }}

        .theme-toggle:hover {{
            background-color: var(--bg-secondary);
            color: var(--text-primary);
        }}

        .content-area {{
            padding: 3rem 2rem 6rem;
            max-width: var(--content-width);
            margin: 0 auto;
        }}

        .content-footer {{
            margin-top: 4rem;
            padding-top: 2rem;
            border-top: 1px solid var(--border-color);
            text-align: center;
            color: var(--text-secondary);
            font-size: 0.875rem;
        }}

        .content-footer a {{
            color: var(--text-primary);
            text-decoration: none;
        }}

        @media (max-width: 768px) {{
            .sidebar {{
                transform: translateX(-100%);
                box-shadow: 0 0 20px rgba(0,0,0,0.1);
            }}

            .sidebar.sidebar-open {{
                transform: translateX(0);
            }}

            .main-content {{
                margin-left: 0;
            }}

            .menu-toggle {{
                display: block;
            }}

            .content-area {{
                padding: 2rem 1.5rem;
            }}

            .search-box {{
                position: static !important;
            }}

            .search-input:focus {{
                width: calc(100% - 6.5rem);
                max-width: none !important;
                position: absolute;
                left: 1rem;
                top: 50%;
                transform: translateY(-50%);
                z-index: 100;
                box-sizing: border-box;
            }}
        }}

        /* Hero Section */
        .hero-section {{
            text-align: center;
            padding: 6rem 1rem 4rem;
            max-width: 800px;
            margin: 0 auto;
        }}
        
        .hero-title {{
            font-size: 3.5rem;
            font-weight: 800;
            margin-bottom: 1rem;
            letter-spacing: -0.03em;
            background: linear-gradient(135deg, var(--text-primary) 0%, var(--text-secondary) 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }}
        
        .hero-subtitle {{
            font-size: 1.25rem;
            color: var(--text-secondary);
            line-height: 1.5;
            max-width: 600px;
            margin: 0 auto 2rem;
        }}

        .content-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
            gap: 1.5rem;
            margin-top: 2rem;
        }}

        .content-card {{
            background-color: var(--bg-primary);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 1.5rem;
            transition: all 0.2s ease;
            display: flex;
            flex-direction: column;
            height: 100%;
        }}
        
        .content-card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 10px 30px -10px rgba(0, 0, 0, 0.1);
            border-color: var(--text-secondary);
        }}
        
        .content-card h3 {{
            margin: 0 0 0.5rem 0;
            font-size: 1.1rem;
            font-weight: 600;
        }}
        
        .content-card h3 a {{
            color: var(--text-primary);
            text-decoration: none;
        }}
        
        .content-card h3 a::after {{
            content: '';
            position: absolute;
            inset: 0;
        }}
        
        .content-card p {{
            margin: 0 0 1rem 0;
            color: var(--text-secondary);
            font-size: 0.9rem;
            flex-grow: 1;
            line-height: 1.5;
        }}
        
        .content-meta {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            font-size: 0.75rem;
            color: var(--text-secondary);
            margin-top: auto;
            padding-top: 1rem;
            border-top: 1px solid var(--border-color);
        }}
        
        .content-type {{
            font-weight: 500;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}
        
        .tags {{
            background-color: var(--bg-secondary);
            padding: 0.2em 0.5em;
            border-radius: 4px;
        }}
    </style>
    <script>
        // Theme initialization
        (function() {{
            const savedTheme = localStorage.getItem('theme');
            const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
            if (savedTheme === 'dark' || (!savedTheme && systemDark)) {{
                document.documentElement.setAttribute('data-theme', 'dark');
            }} else {{
                document.documentElement.setAttribute('data-theme', 'light');
            }}
        }})();
    </script>
</head>
<body>
    <div class="book-container">
        <!-- Sidebar Navigation -->
        <nav class="sidebar" id="sidebar">
            <div class="sidebar-header">
                <a href="/" class="sidebar-title">{}</a>
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
                    <a href="/" class="breadcrumb-home" aria-label="Home"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path><polyline points="9 22 9 12 15 12 15 22"></polyline></svg></a>
                </div>
                <div class="right-actions">
                    <div class="search-box" id="search-container">
                        <input type="text" id="search-input" class="search-input" placeholder="Search documentation...">
                        <div id="search-results" class="search-results"></div>
                    </div>
                    <button class="theme-toggle" id="theme-toggle" aria-label="Toggle theme">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <circle cx="12" cy="12" r="5"></circle>
                            <line x1="12" y1="1" x2="12" y2="3"></line>
                            <line x1="12" y1="21" x2="12" y2="23"></line>
                            <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                            <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                            <line x1="1" y1="12" x2="3" y2="12"></line>
                            <line x1="21" y1="12" x2="23" y2="12"></line>
                            <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                            <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
                        </svg>
                    </button>
                </div>
            </header>

            <!-- Content Area -->
            <main class="content-area">
                <div class="hero-section">
                    <h1 class="hero-title">{}</h1>
                    <p class="hero-subtitle">{}</p>
                </div>

                <div class="content-grid">
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
    </script>
    <script src="/assets/search.js"></script>
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

        let mut breadcrumb = String::from(r#"<a href="/" class="breadcrumb-home" aria-label="Home"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path><polyline points="9 22 9 12 15 12 15 22"></polyline></svg></a>"#);

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
                    r#"<a href="/{}" class="nav-button">← {}</a>"#,
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
                    r#"<a href="/{}" class="nav-button">{} →</a>"#,
                    next_content.slug(),
                    next_content.metadata.get_title()
                ));
            }
        }

        buttons
    }
}
