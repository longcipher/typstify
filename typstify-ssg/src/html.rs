pub mod opengraph_template {
    use leptos::prelude::*;

    #[component]
    pub fn Home(
        logo: &'static str,
        website_name: &'static str,
        website_tagline: Option<&'static str>,
        absolute_url: String,
    ) -> impl IntoView {
        view! {
          <html lang="en" data-theme="light">
            <head>
              <meta charset="utf-8" />
              <meta name="viewport" content="width=device-width, initial-scale=1" />
              <title>{website_name}</title>
              <link rel="stylesheet" href="opengraph_style.css" />
            </head>
            <body class="bg-base-100">
              <div class="min-h-screen hero">
                <div class="text-center hero-content">
                  <div class="max-w-md">
                    <img src=logo alt="Logo" class="mx-auto mb-8 w-32 h-32" />
                    <h1 class="text-5xl font-bold text-primary">{website_name}</h1>
                    {website_tagline
                      .map(|tagline| {
                        view! { <p class="py-6 text-xl text-base-content/70">{tagline}</p> }
                      })}
                  </div>
                </div>
              </div>
            </body>
          </html>
        }
    }

    pub fn home(
        logo: &'static str,
        website_name: &'static str,
        website_tagline: Option<&'static str>,
        absolute_url: String,
    ) -> AnyView {
        view! {
          <Home
            logo=logo
            website_name=website_name
            website_tagline=website_tagline
            absolute_url=absolute_url
          />
        }.into_any()
    }

    #[component]
    pub fn ContentPage(
        title: String,
        logo: &'static str,
        website_name: &'static str,
        absolute_url: String,
    ) -> impl IntoView {
        view! {
          <html lang="en" data-theme="light">
            <head>
              <meta charset="utf-8" />
              <meta name="viewport" content="width=device-width, initial-scale=1" />
              <title>{format!("{} - {}", title, website_name)}</title>
              <link rel="stylesheet" href="../opengraph_style.css" />
            </head>
            <body class="bg-base-100">
              <div class="min-h-screen hero">
                <div class="text-center hero-content">
                  <div class="max-w-4xl">
                    <img src=format!("../{}", logo) alt="Logo" class="mx-auto mb-6 w-24 h-24" />
                    <h1 class="mb-4 text-4xl font-bold text-primary">{title}</h1>
                    <p class="text-lg text-base-content/70">{website_name}</p>
                  </div>
                </div>
              </div>
            </body>
          </html>
        }
    }

    pub fn content(
        title: String,
        logo: &'static str,
        website_name: &'static str,
        absolute_url: String,
    ) -> AnyView {
        view! { <ContentPage title=title logo=logo website_name=website_name absolute_url=absolute_url /> }.into_any()
    }
}
