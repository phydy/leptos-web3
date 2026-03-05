use leptos::{either::EitherOf3, prelude::*};
use leptos_router::{components::Redirect};
use crate::auth::context::use_auth;

/// Wrap any page component with this to require authentication.
///
/// ```rust
/// // In your route definitions:
/// <Route path="/dashboard" view=|| view! {
///     <RequireAuth>
///         <DashboardPage />
///     </RequireAuth>
/// } />
/// ```
///
/// Behaviour:
/// - While Privy is still initialising   → shows a full-page loader
/// - Authenticated                        → renders children normally
/// - Not authenticated                    → redirects to `redirect_to` (default: "/")
#[component]
pub fn RequireAuth(
    children: ChildrenFn,
    /// Where to send unauthenticated visitors. Defaults to "/".
    #[prop(default = "/".to_string())]
    redirect_to: String,
) -> impl IntoView {
    let auth = use_auth();
    view! {

        {move || { if  auth.loading.get() {
                EitherOf3::A( view! {
                    <FullPageLoader />
                })
            } else if auth.authenticated.get() {
                EitherOf3::B(children())
            } else {
                let path = redirect_to.clone();
                EitherOf3::C( view! {

                    <Redirect path=path />
                } )
            }
        }
    }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

#[component]
fn FullPageLoader() -> impl IntoView {
    view! {
        <div class="full-page-loader">
            <div class="spinner" />
            <p>"Checking session…"</p>
        </div>
    }
}