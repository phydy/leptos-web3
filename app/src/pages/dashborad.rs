use leptos::either::{Either, EitherOf3};
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::auth::context::{use_auth, WalletType};
use crate::privy::{
    privy_get_evm_address, privy_get_solana_address,
    sign_message_privy_evm_js, send_eth_privy_js, send_erc20_privy_js, call_contract_privy_js,
    sign_message_privy_solana_js, send_sol_privy_js, send_spl_token_privy_js, invoke_program_privy_js,
    sign_message_metamask_js, send_eth_metamask_js, send_erc20_metamask_js, call_contract_metamask_js,
    sign_message_phantom_js, send_sol_phantom_js, send_spl_token_phantom_js, invoke_program_phantom_js,
    // Counter Contract — EVM
    get_counter_address_js,
    read_counter_evm_js, counter_inc_metamask_js, counter_inc_by_metamask_js,
    counter_inc_privy_js, counter_inc_by_privy_js,
    // Counter Program — Solana
    read_counter_solana_js, counter_initialize_phantom_js, counter_increment_phantom_js,
    counter_initialize_privy_js, counter_increment_privy_js,
};

const SOLANA_COUNTER_PROGRAM_ID: &str = "BHqZnZvzQ9ogmBJKjSzKEukqwxXCrgWgqsaDNbU6QkGw";
use crate::components::protect::RequireAuth;

// ---------------------------------------------------------------------------
// Page root
// ---------------------------------------------------------------------------

#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <RequireAuth>
            <DashboardInner />
        </RequireAuth>
    }
}

// ---------------------------------------------------------------------------
// Inner
// ---------------------------------------------------------------------------

#[component]
fn DashboardInner() -> impl IntoView {
    let auth = use_auth();

    // Re-read Privy embedded wallet addresses reactively.
    // These JS calls return None until the Privy SDK has hydrated the session
    // (which happens after auth.user is set). Wrapping in Memo::new makes them
    // re-evaluate every time auth.user changes, so the panels appear once the
    // session settles — instead of being frozen at the None they return on
    // first render before hydration completes.
    let privy_evm_addr = Memo::new(move |_| {
        let _ = auth.user.get(); // subscribe so memo re-runs when user changes
        let add = privy_get_evm_address();
        log::info!("evm_add: {}", add.clone().is_some());
        add
    });
    let privy_sol_addr = Memo::new(move |_| {
        let _ = auth.user.get();
        let add = privy_get_solana_address();
        log::info!("evm_add: {}", add.clone().is_some());
        add
    });

    // Flat String signals used as props for the Privy EVM panels.
    let evm_addr_sig = Signal::derive(move || privy_evm_addr.get().unwrap_or_default());

    view! {
        <div class="page dashboard-page">
            <h1>"Dashboard"</h1>

            // ── Identity card ──────────────────────────────────────────────
            <div class="user-card">
                <h2>"Session"</h2>
                <dl>
                    {move || auth.user.get().map(|u| view! {
                        <>
                            <dt>"Privy ID"</dt>
                            <dd>{u.id.clone()}</dd>
                        </>
                    })}
                    {move || auth.wallet_address.get().map(|addr| {
                        let label = match auth.wallet_type.get() {
                            Some(WalletType::MetaMask) => "MetaMask",
                            Some(WalletType::Phantom)  => "Phantom",
                            None                       => "Wallet",
                        };
                        view! {
                            <>
                                <dt>{label}" address"</dt>
                                <dd class="monospace">{addr}</dd>
                            </>
                        }
                    })}
                    {move || privy_evm_addr.get().map(|addr| view! {
                        <>
                            <dt>"Privy EVM wallet"</dt>
                            <dd class="monospace">{addr}</dd>
                        </>
                    })}
                    {move || privy_sol_addr.get().map(|addr| view! {
                        <>
                            <dt>"Privy Solana wallet"</dt>
                            <dd class="monospace">{addr}</dd>
                        </>
                    })}
                </dl>
            </div>

            // ── Browser wallet panels ──────────────────────────────────────
            {move || match auth.wallet_type.get() {
                Some(WalletType::MetaMask) => EitherOf3::A(view! {
                    <div class="wallet-section">
                        <p class="panels-heading">"🦊 MetaMask"</p>
                        <div class="action-panels">
                            <SignMessageEth />
                            <SendEthPanel />
                            <SendERC20Panel />
                            <CallContractPanel />
                            <CounterEvmPanel />
                        </div>
                    </div>
                }),
                Some(WalletType::Phantom) => EitherOf3::B(view! {
                    <div class="wallet-section">
                        <p class="panels-heading">"👻 Phantom"</p>
                        <div class="action-panels">
                            <SignMessageSol />
                            <SendSolPanel />
                            <SendSPLPanel />
                            <InvokeProgramPanel />
                            <CounterSolanaPanel />
                        </div>
                    </div>
                }),
                None => EitherOf3::C(view! { <div /> }),
            }}

            // ── Privy embedded EVM wallet panels ───────────────────────────
            // move || re-evaluates each time privy_evm_addr updates.
            {move || match privy_evm_addr.get() {
                Some(_) => Either::Left(view! {
                    <div class="wallet-section">
                        <p class="panels-heading">"🔐 Privy EVM Wallet"</p>
                        <div class="action-panels">
                            <PrivySignMessageEvm addr=evm_addr_sig />
                            <PrivySendEthPanel   addr=evm_addr_sig />
                            <PrivySendERC20Panel addr=evm_addr_sig />
                            <PrivyCallContractPanel addr=evm_addr_sig />
                            <PrivyCounterEvmPanel addr=evm_addr_sig />
                        </div>
                    </div>
                }),
                None => Either::Right(view! { <div /> }),
            }}

            // ── Privy embedded Solana wallet panels ────────────────────────
            {move || match privy_sol_addr.get() {
                Some(_) => Either::Left(view! {
                    <div class="wallet-section">
                        <p class="panels-heading">"🔐 Privy Solana Wallet"</p>
                        <div class="action-panels">
                            <PrivySignMessageSol />
                            <PrivySendSolPanel />
                            <PrivySendSPLPanel />
                            <PrivyInvokeProgramPanel />
                            <PrivyCounterSolanaPanel />
                        </div>
                    </div>
                }),
                None => Either::Right(view! { <div /> }),
            }}

            // ── Fallback when nothing is connected ─────────────────────────
            {move || {
                let no_browser_wallet = auth.wallet_type.get().is_none();
                let no_privy_wallets  = privy_evm_addr.get().is_none()
                                     && privy_sol_addr.get().is_none();
                if no_browser_wallet && no_privy_wallets {
                    Either::Left(view! {
                        <div class="no-wallet-notice">
                            <p>"👛 Connect a wallet or sign in with email to use on-chain features."</p>
                        </div>
                    })
                } else {
                    Either::Right(view! { <div /> })
                }
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

#[component]
fn ActionResult(
    result: ReadSignal<Option<String>>,
    error:  ReadSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <Show when=move || result.get().is_some()>
            <div class="action-result success">
                <span class="label">"Result: "</span>
                <span class="monospace">{move || result.get().unwrap_or_default()}</span>
            </div>
        </Show>
        <Show when=move || error.get().is_some()>
            <div class="action-result error">
                <span class="label">"Error: "</span>
                <span>{move || error.get().unwrap_or_default()}</span>
            </div>
        </Show>
    }
}

// ---------------------------------------------------------------------------
// Ethereum / MetaMask panels
// ---------------------------------------------------------------------------

#[component]
fn SignMessageEth() -> impl IntoView {
    let auth    = use_auth();
    let message = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_sign = move |_| {
        let msg  = message.get();
        let addr = auth.wallet_address.get().unwrap_or_default();
        if msg.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = sign_message_metamask_js(&msg, &addr);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel">
            <h3>"✍️ Sign Message"</h3>
            <textarea class="action-input" placeholder="Message to sign…"
                on:input=move |e| message.set(event_target_value(&e)) />
            <button class="action-btn" on:click=on_sign disabled=move || busy.get()>
                {move || if busy.get() { "Signing…" } else { "Sign with MetaMask" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

#[component]
fn SendEthPanel() -> impl IntoView {
    let auth   = use_auth();
    let to     = RwSignal::new(String::new());
    let amount = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_send = move |_| {
        let from = auth.wallet_address.get().unwrap_or_default();
        let to_v = to.get(); let amt = amount.get();
        if to_v.is_empty() || amt.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = send_eth_metamask_js(&from, &to_v, &amt);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel">
            <h3>"💸 Send ETH"</h3>
            <input class="action-input" type="text" placeholder="Recipient address (0x…)"
                on:input=move |e| to.set(event_target_value(&e)) />
            <input class="action-input" type="text" placeholder="Amount (ETH, e.g. 0.01)"
                on:input=move |e| amount.set(event_target_value(&e)) />
            <button class="action-btn" on:click=on_send disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Send ETH" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

#[component]
fn SendERC20Panel() -> impl IntoView {
    let auth    = use_auth();
    let token   = RwSignal::new(String::new());
    let to      = RwSignal::new(String::new());
    let amount  = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_send = move |_| {
        let from  = auth.wallet_address.get().unwrap_or_default();
        let tok_v = token.get(); let to_v = to.get(); let amt = amount.get();
        if tok_v.is_empty() || to_v.is_empty() || amt.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = send_erc20_metamask_js(&from, &tok_v, &to_v, &amt);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel">
            <h3>"🪙 Send ERC-20 Token"</h3>
            <input class="action-input" type="text" placeholder="Token contract address (0x…)"
                on:input=move |e| token.set(event_target_value(&e)) />
            <input class="action-input" type="text" placeholder="Recipient address (0x…)"
                on:input=move |e| to.set(event_target_value(&e)) />
            <input class="action-input" type="text" placeholder="Amount in raw units (e.g. 1000000 = 1 USDC)"
                on:input=move |e| amount.set(event_target_value(&e)) />
            <button class="action-btn" on:click=on_send disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Send ERC-20" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

#[component]
fn CallContractPanel() -> impl IntoView {
    let auth     = use_auth();
    let contract = RwSignal::new(String::new());
    let data     = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_call = move |_| {
        let from = auth.wallet_address.get().unwrap_or_default();
        let c_v  = contract.get(); let d_v = data.get();
        if c_v.is_empty() || d_v.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = call_contract_metamask_js(&from, &c_v, &d_v);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel">
            <h3>"⚙️ Call Contract"</h3>
            <input class="action-input" type="text" placeholder="Contract address (0x…)"
                on:input=move |e| contract.set(event_target_value(&e)) />
            <textarea class="action-input" placeholder="ABI-encoded calldata (0x-prefixed hex)"
                on:input=move |e| data.set(event_target_value(&e)) />
            <button class="action-btn" on:click=on_call disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Send Transaction" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

// ---------------------------------------------------------------------------
// Solana / Phantom panels
// ---------------------------------------------------------------------------

#[component]
fn SignMessageSol() -> impl IntoView {
    let message = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_sign = move |_| {
        let msg = message.get();
        if msg.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = sign_message_phantom_js(&msg);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel">
            <h3>"✍️ Sign Message"</h3>
            <textarea class="action-input" placeholder="Message to sign…"
                on:input=move |e| message.set(event_target_value(&e)) />
            <button class="action-btn" on:click=on_sign disabled=move || busy.get()>
                {move || if busy.get() { "Signing…" } else { "Sign with Phantom" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

#[component]
fn SendSolPanel() -> impl IntoView {
    let to     = RwSignal::new(String::new());
    let amount = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_send = move |_| {
        let to_v = to.get(); let amt = amount.get();
        if to_v.is_empty() || amt.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = send_sol_phantom_js(&to_v, &amt);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel">
            <h3>"💸 Send SOL"</h3>
            <input class="action-input" type="text" placeholder="Recipient address (base58)"
                on:input=move |e| to.set(event_target_value(&e)) />
            <input class="action-input" type="text" placeholder="Amount (SOL, e.g. 0.01)"
                on:input=move |e| amount.set(event_target_value(&e)) />
            <button class="action-btn" on:click=on_send disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Send SOL" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

#[component]
fn SendSPLPanel() -> impl IntoView {
    let mint   = RwSignal::new(String::new());
    let to     = RwSignal::new(String::new());
    let amount = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_send = move |_| {
        let mint_v = mint.get(); let to_v = to.get(); let amt = amount.get();
        if mint_v.is_empty() || to_v.is_empty() || amt.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = send_spl_token_phantom_js(&to_v, &mint_v, &amt);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel">
            <h3>"🪙 Send SPL Token"</h3>
            <input class="action-input" type="text" placeholder="Mint address (base58)"
                on:input=move |e| mint.set(event_target_value(&e)) />
            <input class="action-input" type="text" placeholder="Recipient address (base58)"
                on:input=move |e| to.set(event_target_value(&e)) />
            <input class="action-input" type="text" placeholder="Amount in raw units (e.g. 1000000 = 1 USDC)"
                on:input=move |e| amount.set(event_target_value(&e)) />
            <button class="action-btn" on:click=on_send disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Send SPL Token" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

#[component]
fn InvokeProgramPanel() -> impl IntoView {
    let program  = RwSignal::new(String::new());
    let data     = RwSignal::new(String::new());
    let accounts = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_invoke = move |_| {
        let prog = program.get(); let d_v = data.get(); let acc_raw = accounts.get();
        if prog.is_empty() || d_v.is_empty() { return; }
        let acc_list: Vec<String> = acc_raw.lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let accounts_js = serde_wasm_bindgen::to_value(&acc_list)
            .unwrap_or(wasm_bindgen::JsValue::NULL);
        let promise = invoke_program_phantom_js(&prog, &d_v, &accounts_js);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel">
            <h3>"⚙️ Invoke Program"</h3>
            <input class="action-input" type="text" placeholder="Program ID (base58)"
                on:input=move |e| program.set(event_target_value(&e)) />
            <textarea class="action-input" placeholder="Instruction data (hex, no 0x)"
                on:input=move |e| data.set(event_target_value(&e)) />
            <textarea class="action-input" placeholder="Account pubkeys — one per line (first = signer/writable)"
                on:input=move |e| accounts.set(event_target_value(&e)) />
            <button class="action-btn" on:click=on_invoke disabled=move || busy.get()>
                {move || if busy.get() { "Invoking…" } else { "Invoke Program" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

// ---------------------------------------------------------------------------
// Privy embedded EVM wallet panels
// addr is Signal<String> so the value is read reactively on each click,
// not captured once at construction time when it may still be empty.
// ---------------------------------------------------------------------------

#[component]
fn PrivySignMessageEvm(addr: Signal<String>) -> impl IntoView {
    let message = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_sign = move |_| {
        let msg  = message.get();
        let addr = addr.get();
        if msg.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = sign_message_privy_evm_js(&msg, &addr);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel privy-panel">
            <h3>"✍️ Sign Message"</h3>
            <textarea class="action-input" placeholder="Message to sign…"
                on:input=move |e| message.set(event_target_value(&e)) />
            <button class="action-btn privy-btn" on:click=on_sign disabled=move || busy.get()>
                {move || if busy.get() { "Signing…" } else { "Sign (Privy EVM)" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

#[component]
fn PrivySendEthPanel(addr: Signal<String>) -> impl IntoView {
    let to     = RwSignal::new(String::new());
    let amount = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_send = move |_| {
        let from = addr.get();
        let to_v = to.get(); let amt = amount.get();
        if to_v.is_empty() || amt.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = send_eth_privy_js(&from, &to_v, &amt);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel privy-panel">
            <h3>"💸 Send ETH"</h3>
            <input class="action-input" type="text" placeholder="Recipient (0x…)"
                on:input=move |e| to.set(event_target_value(&e)) />
            <input class="action-input" type="text" placeholder="Amount (ETH, e.g. 0.01)"
                on:input=move |e| amount.set(event_target_value(&e)) />
            <button class="action-btn privy-btn" on:click=on_send disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Send ETH (Privy)" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

#[component]
fn PrivySendERC20Panel(addr: Signal<String>) -> impl IntoView {
    let token  = RwSignal::new(String::new());
    let to     = RwSignal::new(String::new());
    let amount = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_send = move |_| {
        let from  = addr.get();
        let tok_v = token.get(); let to_v = to.get(); let amt = amount.get();
        if tok_v.is_empty() || to_v.is_empty() || amt.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = send_erc20_privy_js(&from, &tok_v, &to_v, &amt);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel privy-panel">
            <h3>"🪙 Send ERC-20"</h3>
            <input class="action-input" type="text" placeholder="Token contract (0x…)"
                on:input=move |e| token.set(event_target_value(&e)) />
            <input class="action-input" type="text" placeholder="Recipient (0x…)"
                on:input=move |e| to.set(event_target_value(&e)) />
            <input class="action-input" type="text" placeholder="Amount in raw units"
                on:input=move |e| amount.set(event_target_value(&e)) />
            <button class="action-btn privy-btn" on:click=on_send disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Send ERC-20 (Privy)" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

#[component]
fn PrivyCallContractPanel(addr: Signal<String>) -> impl IntoView {
    let contract = RwSignal::new(String::new());
    let data     = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_call = move |_| {
        let from = addr.get();
        let c_v  = contract.get(); let d_v = data.get();
        if c_v.is_empty() || d_v.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = call_contract_privy_js(&from, &c_v, &d_v);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel privy-panel">
            <h3>"⚙️ Call Contract"</h3>
            <input class="action-input" type="text" placeholder="Contract address (0x…)"
                on:input=move |e| contract.set(event_target_value(&e)) />
            <textarea class="action-input" placeholder="ABI-encoded calldata (0x-prefixed hex)"
                on:input=move |e| data.set(event_target_value(&e)) />
            <button class="action-btn privy-btn" on:click=on_call disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Send Tx (Privy)" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

// ---------------------------------------------------------------------------
// Privy embedded Solana wallet panels
// ---------------------------------------------------------------------------

#[component]
fn PrivySignMessageSol() -> impl IntoView {
    let message = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_sign = move |_| {
        let msg = message.get();
        if msg.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = sign_message_privy_solana_js(&msg);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel privy-panel">
            <h3>"✍️ Sign Message"</h3>
            <textarea class="action-input" placeholder="Message to sign…"
                on:input=move |e| message.set(event_target_value(&e)) />
            <button class="action-btn privy-btn" on:click=on_sign disabled=move || busy.get()>
                {move || if busy.get() { "Signing…" } else { "Sign (Privy Solana)" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

#[component]
fn PrivySendSolPanel() -> impl IntoView {
    let to     = RwSignal::new(String::new());
    let amount = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_send = move |_| {
        let to_v = to.get(); let amt = amount.get();
        if to_v.is_empty() || amt.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = send_sol_privy_js(&to_v, &amt);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel privy-panel">
            <h3>"💸 Send SOL"</h3>
            <input class="action-input" type="text" placeholder="Recipient (base58)"
                on:input=move |e| to.set(event_target_value(&e)) />
            <input class="action-input" type="text" placeholder="Amount (SOL, e.g. 0.01)"
                on:input=move |e| amount.set(event_target_value(&e)) />
            <button class="action-btn privy-btn" on:click=on_send disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Send SOL (Privy)" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

#[component]
fn PrivySendSPLPanel() -> impl IntoView {
    let mint   = RwSignal::new(String::new());
    let to     = RwSignal::new(String::new());
    let amount = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_send = move |_| {
        let mint_v = mint.get(); let to_v = to.get(); let amt = amount.get();
        if mint_v.is_empty() || to_v.is_empty() || amt.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = send_spl_token_privy_js(&to_v, &mint_v, &amt);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel privy-panel">
            <h3>"🪙 Send SPL Token"</h3>
            <input class="action-input" type="text" placeholder="Mint address (base58)"
                on:input=move |e| mint.set(event_target_value(&e)) />
            <input class="action-input" type="text" placeholder="Recipient (base58)"
                on:input=move |e| to.set(event_target_value(&e)) />
            <input class="action-input" type="text" placeholder="Amount in raw units"
                on:input=move |e| amount.set(event_target_value(&e)) />
            <button class="action-btn privy-btn" on:click=on_send disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Send SPL (Privy)" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

#[component]
fn PrivyInvokeProgramPanel() -> impl IntoView {
    let program  = RwSignal::new(String::new());
    let data     = RwSignal::new(String::new());
    let accounts = RwSignal::new(String::new());
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_invoke = move |_| {
        let prog = program.get(); let d_v = data.get(); let acc_raw = accounts.get();
        if prog.is_empty() || d_v.is_empty() { return; }
        let acc_list: Vec<String> = acc_raw.lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let accounts_js = serde_wasm_bindgen::to_value(&acc_list)
            .unwrap_or(wasm_bindgen::JsValue::NULL);
        let promise = invoke_program_privy_js(&prog, &d_v, &accounts_js);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel privy-panel">
            <h3>"⚙️ Invoke Program"</h3>
            <input class="action-input" type="text" placeholder="Program ID (base58)"
                on:input=move |e| program.set(event_target_value(&e)) />
            <textarea class="action-input" placeholder="Instruction data (hex, no 0x)"
                on:input=move |e| data.set(event_target_value(&e)) />
            <textarea class="action-input" placeholder="Account pubkeys — one per line"
                on:input=move |e| accounts.set(event_target_value(&e)) />
            <button class="action-btn privy-btn" on:click=on_invoke disabled=move || busy.get()>
                {move || if busy.get() { "Invoking…" } else { "Invoke (Privy)" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

// ---------------------------------------------------------------------------
// Counter Contract — EVM panels
// ---------------------------------------------------------------------------

#[component]
fn CounterEvmPanel() -> impl IntoView {
    let auth     = use_auth();
    let contract = RwSignal::new(String::new());
    let by       = RwSignal::new(String::new());
    let (count,  set_count)  = signal(None::<String>);
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    // Auto-fill the deployed address for the connected chain on mount.
    spawn_local(async move {
        if let Ok(v) = wasm_bindgen_futures::JsFuture::from(get_counter_address_js()).await {
            if let Some(addr) = v.as_string() {
                contract.set(addr);
            }
        }
    });

    let on_read = move |_| {
        let addr = contract.get();
        if addr.is_empty() { return; }
        set_busy.set(true); set_error.set(None);
        let promise = read_counter_evm_js(&addr);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_count.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    let on_inc = move |_| {
        let from = auth.wallet_address.get().unwrap_or_default();
        let addr = contract.get();
        if addr.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = counter_inc_metamask_js(&from, &addr);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    let on_inc_by = move |_| {
        let from = auth.wallet_address.get().unwrap_or_default();
        let addr = contract.get();
        let by_v = by.get();
        if addr.is_empty() || by_v.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = counter_inc_by_metamask_js(&from, &addr, &by_v);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel">
            <h3>"🔢 Counter Contract"</h3>
            <input class="action-input" type="text" placeholder="Counter contract address (0x…)"
                on:input=move |e| contract.set(event_target_value(&e)) />
            <Show when=move || count.get().is_some()>
                <div class="action-result success">
                    <span class="label">"Count: "</span>
                    <span class="monospace">{move || count.get().unwrap_or_default()}</span>
                </div>
            </Show>
            <button class="action-btn" on:click=on_read disabled=move || busy.get()>
                {move || if busy.get() { "Loading…" } else { "Read Counter" }}
            </button>
            <button class="action-btn" on:click=on_inc disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Increment (inc)" }}
            </button>
            <input class="action-input" type="number" placeholder="Increment by (e.g. 5)"
                on:input=move |e| by.set(event_target_value(&e)) />
            <button class="action-btn" on:click=on_inc_by disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Increment By" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

#[component]
fn CounterSolanaPanel() -> impl IntoView {
    let program  = RwSignal::new(SOLANA_COUNTER_PROGRAM_ID.to_string());
    let (count,  set_count)  = signal(None::<String>);
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_read = move |_| {
        let prog = program.get();
        if prog.is_empty() { return; }
        set_busy.set(true); set_error.set(None);
        let promise = read_counter_solana_js(&prog);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_count.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    let on_init = move |_| {
        let prog = program.get();
        if prog.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = counter_initialize_phantom_js(&prog);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    let on_increment = move |_| {
        let prog = program.get();
        if prog.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = counter_increment_phantom_js(&prog);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel">
            <h3>"🔢 Solana Counter"</h3>
            <input class="action-input" type="text" placeholder="Program ID (base58)"
                prop:value=move || program.get()
                on:input=move |e| program.set(event_target_value(&e)) />
            <Show when=move || count.get().is_some()>
                <div class="action-result success">
                    <span class="label">"Count: "</span>
                    <span class="monospace">{move || count.get().unwrap_or_default()}</span>
                </div>
            </Show>
            <button class="action-btn" on:click=on_read disabled=move || busy.get()>
                {move || if busy.get() { "Loading…" } else { "Read Counter" }}
            </button>
            <button class="action-btn" on:click=on_init disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Initialize Counter" }}
            </button>
            <button class="action-btn" on:click=on_increment disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Increment Counter" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

// ---------------------------------------------------------------------------
// Privy embedded wallet — Counter panels
// ---------------------------------------------------------------------------

#[component]
fn PrivyCounterEvmPanel(addr: Signal<String>) -> impl IntoView {
    let contract = RwSignal::new(String::new());
    let by       = RwSignal::new(String::new());
    let (count,  set_count)  = signal(None::<String>);
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    // Auto-fill the deployed address for the connected chain on mount.
    spawn_local(async move {
        if let Ok(v) = wasm_bindgen_futures::JsFuture::from(get_counter_address_js()).await {
            if let Some(a) = v.as_string() {
                contract.set(a);
            }
        }
    });

    let on_read = move |_| {
        let a = contract.get();
        if a.is_empty() { return; }
        set_busy.set(true); set_error.set(None);
        let promise = read_counter_evm_js(&a);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_count.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    let on_inc = move |_| {
        let from = addr.get();
        let a    = contract.get();
        if a.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = counter_inc_privy_js(&from, &a);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    let on_inc_by = move |_| {
        let from = addr.get();
        let a    = contract.get();
        let by_v = by.get();
        if a.is_empty() || by_v.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = counter_inc_by_privy_js(&from, &a, &by_v);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel privy-panel">
            <h3>"🔢 Counter Contract"</h3>
            <input class="action-input" type="text" placeholder="Counter contract address (0x…)"
                on:input=move |e| contract.set(event_target_value(&e)) />
            <Show when=move || count.get().is_some()>
                <div class="action-result success">
                    <span class="label">"Count: "</span>
                    <span class="monospace">{move || count.get().unwrap_or_default()}</span>
                </div>
            </Show>
            <button class="action-btn privy-btn" on:click=on_read disabled=move || busy.get()>
                {move || if busy.get() { "Loading…" } else { "Read Counter" }}
            </button>
            <button class="action-btn privy-btn" on:click=on_inc disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Increment (Privy)" }}
            </button>
            <input class="action-input" type="number" placeholder="Increment by (e.g. 5)"
                on:input=move |e| by.set(event_target_value(&e)) />
            <button class="action-btn privy-btn" on:click=on_inc_by disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Increment By (Privy)" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

#[component]
fn PrivyCounterSolanaPanel() -> impl IntoView {
    let program  = RwSignal::new(SOLANA_COUNTER_PROGRAM_ID.to_string());
    let (count,  set_count)  = signal(None::<String>);
    let (result, set_result) = signal(None::<String>);
    let (error,  set_error)  = signal(None::<String>);
    let (busy,   set_busy)   = signal(false);

    let on_read = move |_| {
        let prog = program.get();
        if prog.is_empty() { return; }
        set_busy.set(true); set_error.set(None);
        let promise = read_counter_solana_js(&prog);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_count.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    let on_init = move |_| {
        let prog = program.get();
        if prog.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = counter_initialize_privy_js(&prog);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    let on_increment = move |_| {
        let prog = program.get();
        if prog.is_empty() { return; }
        set_busy.set(true); set_result.set(None); set_error.set(None);
        let promise = counter_increment_privy_js(&prog);
        spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(v)  => set_result.set(v.as_string()),
                Err(e) => set_error.set(Some(js_err_str(&e))),
            }
            set_busy.set(false);
        });
    };

    view! {
        <div class="action-panel privy-panel">
            <h3>"🔢 Solana Counter"</h3>
            <input class="action-input" type="text" placeholder="Program ID (base58)"
                prop:value=move || program.get()
                on:input=move |e| program.set(event_target_value(&e)) />
            <Show when=move || count.get().is_some()>
                <div class="action-result success">
                    <span class="label">"Count: "</span>
                    <span class="monospace">{move || count.get().unwrap_or_default()}</span>
                </div>
            </Show>
            <button class="action-btn privy-btn" on:click=on_read disabled=move || busy.get()>
                {move || if busy.get() { "Loading…" } else { "Read Counter" }}
            </button>
            <button class="action-btn privy-btn" on:click=on_init disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Initialize (Privy)" }}
            </button>
            <button class="action-btn privy-btn" on:click=on_increment disabled=move || busy.get()>
                {move || if busy.get() { "Sending…" } else { "Increment (Privy)" }}
            </button>
            <ActionResult result error />
        </div>
    }
}

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

fn js_err_str(e: &wasm_bindgen::JsValue) -> String {
    js_sys::Reflect::get(e, &wasm_bindgen::JsValue::from_str("message"))
        .ok()
        .and_then(|v| v.as_string())
        .or_else(|| e.as_string())
        .unwrap_or_else(|| "Unknown error".into())
}