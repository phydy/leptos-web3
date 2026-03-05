use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

// Modules
pub mod components;
pub mod pages;
pub mod privy;
pub mod auth;

// Top-Level pages
use crate::pages::home::HomePage;
use crate::pages::dashborad::DashboardPage;
use crate::components::nav::Nav;
use crate::auth::context::AuthContextProvider;


/// An app router which renders the homepage and handles 404's
//#[component]
//pub fn App() -> impl IntoView {
//    // Provides context that manages stylesheets, titles, meta tags, etc.
//    provide_meta_context();
//
//    view! {
//        <Html attr:lang="en" attr:dir="ltr" attr:data-theme="light" />
//
//        // sets the document title
//        <Title text="Welcome to Leptos Privy" />
//
//        // injects metadata in the <head> of the page
//        <Meta charset="UTF-8" />
//        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />
//
//        <Router>
//            <Routes fallback=|| view! { NotFound }>
//                <Route path=path!("/") view=HomePage />
//                <Route path=path!("/dashboard") view=DashboardPage />
//            </Routes>
//        </Router>
//    }
//}
//

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        // AuthContextProvider initialises Privy and wraps the whole app
        // so every descendant can call use_context::<AuthContext>()
        <AuthContextProvider>
            <Router>
                <Nav />
                <main>
                    <Routes fallback=|| view! { 
                        <div> NotFound </div>
                    }>
                        <Route path=path!("/") view=HomePage/>
                        <Route path=path!("/dashboard") view=DashboardPage  />
                    </Routes>
                </main>
            </Router>
        </AuthContextProvider>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="full-page-loader">
            <div class="spinner" />
            <p>"Not Found!"</p>
        </div>
    }
}