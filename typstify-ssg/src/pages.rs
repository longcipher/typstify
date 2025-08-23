use leptos::prelude::*;
use crate::config::BuildConfig;
use crate::content::{Content, GenerateHtmlError};

#[component]
pub fn Layout(
    config: BuildConfig<'static>,
    children: Children,
    additional_js: Option<AnyView>,
) -> impl IntoView {
    view! {
        <html lang="en" data-theme="light">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <title>{config.website_name}</title>
                <link rel="stylesheet" href="/style/main.css"/>
                <link rel="icon" href="/favicon.ico"/>
            </head>
            <body class="bg-base-100 text-base-content">
                <div class="min-h-screen flex flex-col">
                    <header class="navbar bg-base-200 shadow-lg">
                        <div class="navbar-start">
                            <a class="btn btn-ghost text-xl" href="/">
                                <img src={config.logo} alt="Logo" class="w-8 h-8 mr-2"/>
                                {config.website_name}
                            </a>
                        </div>
                        <div class="navbar-end">
                            <label class="swap swap-rotate">
                                <input type="checkbox" class="theme-controller" value="dark"/>
                                <svg class="swap-off fill-current w-6 h-6" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                                    <path d="M5.64,17l-.71.71a1,1,0,0,0,0,1.41,1,1,0,0,0,1.41,0l.71-.71A1,1,0,0,0,5.64,17ZM5,12a1,1,0,0,0-1-1H3a1,1,0,0,0,0,2H4A1,1,0,0,0,5,12Zm7-7a1,1,0,0,0,1-1V3a1,1,0,0,0-2,0V4A1,1,0,0,0,12,5ZM5.64,7.05a1,1,0,0,0,.7.29,1,1,0,0,0,.71-.29,1,1,0,0,0,0-1.41l-.71-.71A1,1,0,0,0,4.93,6.34Zm12,.29a1,1,0,0,0,.7-.29l.71-.71a1,1,0,1,0-1.41-1.41L17,5.64a1,1,0,0,0,0,1.41A1,1,0,0,0,17.66,7.34ZM21,11H20a1,1,0,0,0,0,2h1a1,1,0,0,0,0-2Zm-9,8a1,1,0,0,0-1,1v1a1,1,0,0,0,2,0V20A1,1,0,0,0,12,19ZM18.36,17A1,1,0,0,0,17,18.36l.71.71a1,1,0,0,0,1.41,0,1,1,0,0,0,0-1.41ZM12,6.5A5.5,5.5,0,1,0,17.5,12,5.51,5.51,0,0,0,12,6.5Zm0,9A3.5,3.5,0,1,1,15.5,12,3.5,3.5,0,0,1,12,15.5Z"/>
                                </svg>
                                <svg class="swap-on fill-current w-6 h-6" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                                    <path d="M21.64,13a1,1,0,0,0-1.05-.14,8.05,8.05,0,0,1-3.37.73A8.15,8.15,0,0,1,9.08,5.49a8.59,8.59,0,0,1,.25-2A1,1,0,0,0,8,2.36,10.14,10.14,0,1,0,22,14.05,1,1,0,0,0,21.64,13Zm-9.5,6.69A8.14,8.14,0,0,1,7.08,5.22v.27A10.15,10.15,0,0,0,17.22,15.63a9.79,9.79,0,0,0,2.1-.22A8.11,8.11,0,0,1,12.14,19.73Z"/>
                                </svg>
                            </label>
                        </div>
                    </header>
                    
                    <main class="flex-1 container mx-auto px-4 py-8">
                        {children()}
                    </main>
                    
                    <footer class="footer footer-center p-10 bg-base-200 text-base-content">
                        <aside>
                            <img src={config.logo} alt="Logo" class="w-12 h-12"/>
                            <p class="font-bold">{config.website_name}</p>
                            {config.website_tagline.map(|tagline| view! {
                                <p>{tagline}</p>
                            })}
                            <p>"Built with Typstify SSG"</p>
                        </aside>
                    </footer>
                </div>
                
                {additional_js}
            </body>
        </html>
    }
}

pub fn index(
    content: &[Content],
    config: BuildConfig<'static>,
    additional_js: Option<AnyView>,
) -> AnyView {
    view! {
        <Layout config=config additional_js=additional_js>
            <div class="hero bg-gradient-to-r from-primary to-secondary text-primary-content mb-16 rounded-box">
                <div class="hero-content text-center py-20">
                    <div class="max-w-md">
                        <h1 class="mb-5 text-5xl font-bold">{config.website_name}</h1>
                        {if !config.website_tagline.is_empty() {
                            view! {
                                <p class="mb-5 text-xl">{config.website_tagline}</p>
                            }.into_any()
                        } else {
                            view! { }.into_any()
                        }}
                        <a href="#content" class="btn btn-accent">
                            "Get Started"
                        </a>
                    </div>
                </div>
            </div>
            
            <section id="content">
                <h2 class="text-3xl font-bold text-center mb-12">"Latest Content"</h2>
                <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                    {content.iter().take(6).map(|content_item| {
                        view! {
                            <div class="card bg-base-200 shadow-xl">
                                <div class="card-body">
                                    <h3 class="card-title">{content_item.meta().get_title()}</h3>
                                    {let desc = content_item.meta().get_description();
                                    if !desc.is_empty() {
                                        view! {
                                            <p class="text-base-content/70">{desc}</p>
                                        }.into_any()
                                    } else {
                                        view! { }.into_any()
                                    }}
                                    <div class="card-actions justify-end">
                                        <a href={format!("/{}", content_item.slug())} class="btn btn-primary">
                                            "Read More"
                                        </a>
                                    </div>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </section>
        </Layout>
    }.into_any()
}

pub fn content(
    content_item: &Content,
    config: BuildConfig<'static>,
    additional_js: Option<AnyView>,
) -> Result<AnyView, GenerateHtmlError> {
    let html_content = content_item.generate_html()?;
    
    Ok(view! {
        <Layout config=config additional_js=additional_js>
            <article class="prose prose-lg max-w-4xl mx-auto">
                <header class="mb-8 text-center">
                    <h1 class="text-4xl font-bold text-primary mb-4">
                        {content_item.meta().get_title()}
                    </h1>
                    
                    <div class="flex flex-wrap justify-center gap-4 text-sm text-base-content/70">
                        {if let Some(author) = content_item.meta().get_author() {
                            view! {
                                <span>"By " {author}</span>
                            }.into_any()
                        } else {
                            view! { }.into_any()
                        }}
                        
                        {if let Some(date) = content_item.meta().get_date() {
                            view! {
                                <span>{date}</span>
                            }.into_any()
                        } else {
                            view! { }.into_any()
                        }}
                        
                        {if !content_item.meta().tags.is_empty() {
                            view! {
                                <div class="flex gap-2">
                                    {content_item.meta().tags.iter().map(|tag| view! {
                                        <span class="badge badge-primary">{tag}</span>
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        } else {
                            view! { }.into_any()
                        }}
                    </div>
                </header>
                
                <div inner_html=html_content></div>
                
                <footer class="mt-12 pt-8 border-t border-base-300">
                    <div class="flex justify-between items-center">
                        <a href="/" class="btn btn-outline">
                            "← Back to Home"
                        </a>
                        
                        <div class="flex gap-2">
                            <button class="btn btn-ghost btn-sm" onclick="window.scrollTo({top: 0, behavior: 'smooth'})">
                                "↑ Top"
                            </button>
                        </div>
                    </div>
                </footer>
            </article>
        </Layout>
    }.into_any())
}

pub fn not_found_page(
    config: BuildConfig<'static>,
    additional_js: Option<AnyView>,
) -> AnyView {
    view! {
        <Layout config=config additional_js=additional_js>
            <div class="hero min-h-96">
                <div class="hero-content text-center">
                    <div class="max-w-md">
                        <h1 class="mb-5 text-5xl font-bold text-error">"404"</h1>
                        <p class="mb-5">"Sorry, the page you are looking for doesn't exist."</p>
                        <a href="/" class="btn btn-primary">
                            "Go Home"
                        </a>
                    </div>
                </div>
            </div>
        </Layout>
    }.into_any()
}
